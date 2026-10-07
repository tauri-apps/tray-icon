// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod util;
mod icon;
mod menu;

use std::{path::Path, thread::JoinHandle};

use crossbeam_channel::{select, unbounded, Receiver, Sender};
use ksni::blocking::{Handle, TrayMethods};
use muda::{ContextMenu, MenuChangeEvent, MenuSnapshotHandle};

use crate::{
    icon::IconType, MouseButton, MouseButtonState, Rect, TrayIconAttributes, TrayIconEvent,
    TrayIconId,
};

pub(crate) use icon::PlatformIcon;

pub struct TrayIcon {
    handle: Handle<StatusNotifierTray>,
    menu_watcher_thread_shutdown_tx: Sender<()>,
    menu_watcher_thread: Option<JoinHandle<()>>,
    #[allow(dead_code)]
    // Keep the menu alive to ensure the snapshot remains valid
    menu: Option<Box<dyn ContextMenu>>,
}

impl TrayIcon {
    pub fn new(id: TrayIconId, attrs: TrayIconAttributes) -> crate::Result<Self> {
        // A custom icon takes precedence over a native icon
        let icon_name = match (&attrs.icon, &attrs.native_icon) {
            (None, Some(icon)) => icon.freedesktop_name().to_owned(),
            _ => String::new(),
        };
        let handle = StatusNotifierTray {
            id,
            icon: attrs
                .icon
                .map(|icon| icon.inner.into())
                .into_iter()
                .collect(),
            icon_name,
            title: attrs.title.unwrap_or_default(),
            tooltip: attrs.tooltip.unwrap_or_default(),
            status: ksni::Status::Active,
            menu_snapshot: attrs.menu.as_ref().map(|menu| menu.snapshot_handle()),
        }
        .assume_sni_available(true)
        .disable_dbus_name(util::is_sandboxed())
        .spawn()?;

        let (menu_watcher_thread_shutdown_tx, menu_watcher_thread_shutdown_rx) = unbounded();

        // Spawn a thread to watch for menu changes and update the tray icon accordingly
        let menu_watcher_thread = std::thread::Builder::new()
            .name("tray-icon-menu-watcher".into())
            .spawn({
                let handle = handle.clone();
                move || watch_menu_changes(handle, menu_watcher_thread_shutdown_rx)
            })?;

        Ok(Self {
            handle,
            menu_watcher_thread_shutdown_tx,
            menu_watcher_thread: Some(menu_watcher_thread),
            menu: attrs.menu,
        })
    }

    pub fn set_icon(&mut self, icon: Option<IconType>) -> crate::Result<()> {
        let _ = self.handle.update(move |tray| tray.set_icon(icon));
        Ok(())
    }

    pub fn set_menu(&mut self, menu: Option<Box<dyn ContextMenu>>) {
        let snapshot = menu.as_ref().map(|menu| menu.snapshot_handle());
        let _ = self
            .handle
            .update(move |tray| tray.menu_snapshot = snapshot);
        self.menu = menu;
    }

    pub fn set_tooltip<S: AsRef<str>>(&mut self, tooltip: Option<S>) -> crate::Result<()> {
        let tooltip = tooltip
            .as_ref()
            .map(|tooltip| tooltip.as_ref())
            .unwrap_or_default()
            .to_string();
        let _ = self.handle.update(move |tray| tray.tooltip = tooltip);
        Ok(())
    }

    pub fn set_title<S: AsRef<str>>(&mut self, title: Option<S>) {
        let title = title
            .as_ref()
            .map(|title| title.as_ref())
            .unwrap_or_default()
            .to_string();
        let _ = self.handle.update(move |tray| tray.title = title);
    }

    pub fn set_visible(&mut self, visible: bool) -> crate::Result<()> {
        let status = if visible {
            ksni::Status::Active
        } else {
            ksni::Status::Passive
        };
        let _ = self.handle.update(move |tray| tray.status = status);
        Ok(())
    }

    pub fn set_temp_dir_path<P: AsRef<Path>>(&mut self, _path: Option<P>) {}

    pub fn rect(&self) -> Option<crate::Rect> {
        None
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        let _ = self.menu_watcher_thread_shutdown_tx.send(());
        if let Some(menu_watcher_thread) = self.menu_watcher_thread.take() {
            let _ = menu_watcher_thread.join();
        }
        self.handle.shutdown().wait();
    }
}

fn watch_menu_changes(handle: Handle<StatusNotifierTray>, shutdown: Receiver<()>) {
    let changes = MenuChangeEvent::receiver();

    loop {
        select! {
            recv(shutdown) -> _ => break,
            recv(changes) -> change => {
                if change.is_err() {
                    break;
                }

                let _ = handle.update(|_| {});
            },
        }
    }
}

pub(super) struct StatusNotifierTray {
    id: TrayIconId,
    icon: Vec<ksni::Icon>,
    icon_name: String,
    title: String,
    tooltip: String,
    status: ksni::Status,
    menu_snapshot: Option<MenuSnapshotHandle>,
}

impl StatusNotifierTray {
    fn set_icon(&mut self, icon: Option<IconType>) {
        self.icon.clear();
        self.icon_name.clear();
        match icon {
            Some(IconType::Custom(icon)) => self.icon.push(icon.inner.into()),
            Some(IconType::Native(icon)) => self.icon_name = icon.freedesktop_name().to_owned(),
            None => {}
        }
    }

    fn emit_click(&self, x: i32, y: i32, button: MouseButton) {
        TrayIconEvent::send(TrayIconEvent::Click {
            id: self.id.clone(),
            position: crate::dpi::PhysicalPosition::new(x as f64, y as f64),
            rect: Rect::default(),
            button,
            button_state: MouseButtonState::Up,
        });
    }
}

impl ksni::Tray for StatusNotifierTray {
    fn id(&self) -> String {
        self.id.0.clone()
    }

    fn activate(&mut self, x: i32, y: i32) {
        self.emit_click(x, y, MouseButton::Left);
    }

    fn secondary_activate(&mut self, x: i32, y: i32) {
        self.emit_click(x, y, MouseButton::Middle);
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn status(&self) -> ksni::Status {
        self.status
    }

    fn icon_name(&self) -> String {
        self.icon_name.clone()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        self.icon.clone()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            icon_pixmap: self.icon.clone(),
            title: self.title.clone(),
            description: self.tooltip.clone(),
            ..Default::default()
        }
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        self.menu_snapshot
            .as_ref()
            .map(|snapshot| menu::into_ksni_menu(&snapshot.items()))
            .unwrap_or_default()
    }

    fn menu_about_to_show(&mut self) {}
}
