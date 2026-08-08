// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Serves the tray over StatusNotifierItem directly.
//!
//! libappindicator never implemented the spec's `Activate` method, so hosts
//! fall back to opening the context menu and left clicks are never reported.
//! Talking the protocol ourselves means `Activate` and `SecondaryActivate`
//! arrive as ordinary click events, matching the other platforms.

mod icon;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::icon::Icon;
pub(crate) use icon::PlatformIcon;

use crate::{MouseButton, MouseButtonState, TrayIconAttributes, TrayIconEvent, TrayIconId};
use ksni::TrayMethods;
use muda::{ContextMenu, MenuId, MenuItemKind};

/// Snapshot of the muda menu, refreshed whenever the menu is replaced.
///
/// ksni asks for the menu on demand and expects to own it, so the muda tree is
/// flattened rather than referenced.
#[derive(Clone, Default)]
struct MenuSnapshot {
    items: Vec<SnapshotItem>,
}

#[derive(Clone)]
enum SnapshotItem {
    Item {
        id: MenuId,
        label: String,
        enabled: bool,
    },
    Check {
        id: MenuId,
        label: String,
        enabled: bool,
        checked: bool,
    },
    Submenu {
        label: String,
        enabled: bool,
        items: Vec<SnapshotItem>,
    },
    Separator,
}

fn snapshot_items(items: Vec<MenuItemKind>) -> Vec<SnapshotItem> {
    items
        .into_iter()
        .map(|item| match item {
            MenuItemKind::MenuItem(i) => SnapshotItem::Item {
                id: i.id().clone(),
                label: i.text(),
                enabled: i.is_enabled(),
            },
            MenuItemKind::Check(i) => SnapshotItem::Check {
                id: i.id().clone(),
                label: i.text(),
                enabled: i.is_enabled(),
                checked: i.is_checked(),
            },
            MenuItemKind::Icon(i) => SnapshotItem::Item {
                id: i.id().clone(),
                label: i.text(),
                enabled: i.is_enabled(),
            },
            MenuItemKind::Submenu(i) => SnapshotItem::Submenu {
                label: i.text(),
                enabled: i.is_enabled(),
                items: snapshot_items(i.items()),
            },
            // Predefined items are largely window-management actions with no
            // StatusNotifierItem equivalent; only the separator carries over.
            MenuItemKind::Predefined(i) => match i.text().as_str() {
                "" => SnapshotItem::Separator,
                text => SnapshotItem::Item {
                    id: i.id().clone(),
                    label: text.to_string(),
                    enabled: true,
                },
            },
        })
        .collect()
}

fn to_ksni_items(items: &[SnapshotItem]) -> Vec<ksni::MenuItem<TrayHandler>> {
    use ksni::menu::{CheckmarkItem, StandardItem, SubMenu};

    items
        .iter()
        .map(|item| match item {
            SnapshotItem::Separator => ksni::MenuItem::Separator,
            SnapshotItem::Item { id, label, enabled } => {
                let id = id.clone();
                StandardItem {
                    label: label.clone(),
                    enabled: *enabled,
                    activate: Box::new(move |_: &mut TrayHandler| {
                        muda::MenuEvent::send(muda::MenuEvent { id: id.clone() })
                    }),
                    ..Default::default()
                }
                .into()
            }
            SnapshotItem::Check {
                id,
                label,
                enabled,
                checked,
            } => {
                let id = id.clone();
                CheckmarkItem {
                    label: label.clone(),
                    enabled: *enabled,
                    checked: *checked,
                    activate: Box::new(move |_: &mut TrayHandler| {
                        muda::MenuEvent::send(muda::MenuEvent { id: id.clone() })
                    }),
                    ..Default::default()
                }
                .into()
            }
            SnapshotItem::Submenu {
                label,
                enabled,
                items,
            } => SubMenu {
                label: label.clone(),
                enabled: *enabled,
                submenu: to_ksni_items(items),
                ..Default::default()
            }
            .into(),
        })
        .collect()
}

struct TrayHandler {
    id: TrayIconId,
    title: String,
    tooltip: String,
    icon_path: Option<PathBuf>,
    icon_theme_path: Option<PathBuf>,
    visible: bool,
    menu: MenuSnapshot,
}

impl TrayHandler {
    fn click(&self, button: MouseButton, x: i32, y: i32) {
        // The protocol reports where the click happened but says nothing about
        // the icon's own geometry, so the rect collapses to that point.
        let position = crate::dpi::PhysicalPosition::new(x as f64, y as f64);

        TrayIconEvent::send(TrayIconEvent::Click {
            id: self.id.clone(),
            position,
            rect: crate::Rect {
                position,
                size: crate::dpi::PhysicalSize::new(0, 0),
            },
            button,
            button_state: MouseButtonState::Up,
        });
    }
}

impl ksni::Tray for TrayHandler {
    fn id(&self) -> String {
        self.id.as_ref().to_string()
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: self.tooltip.clone(),
            ..Default::default()
        }
    }

    fn icon_theme_path(&self) -> String {
        self.icon_theme_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    fn icon_name(&self) -> String {
        self.icon_path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    fn status(&self) -> ksni::Status {
        if self.visible {
            ksni::Status::Active
        } else {
            ksni::Status::Passive
        }
    }

    fn activate(&mut self, x: i32, y: i32) {
        self.click(MouseButton::Left, x, y);
    }

    fn secondary_activate(&mut self, x: i32, y: i32) {
        self.click(MouseButton::Middle, x, y);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        to_ksni_items(&self.menu.items)
    }
}

pub struct TrayIcon {
    id: TrayIconId,
    handle: Option<ksni::Handle<TrayHandler>>,
    state: Arc<Mutex<TrayState>>,
    temp_dir_path: Option<PathBuf>,
    path: PathBuf,
    counter: u32,
    menu: Option<Box<dyn ContextMenu>>,
}

/// Mirrors the handler's fields so updates can be applied without a round trip
/// through the tray task.
#[derive(Default)]
struct TrayState {
    title: String,
    tooltip: String,
    icon_path: Option<PathBuf>,
    icon_theme_path: Option<PathBuf>,
    visible: bool,
    menu: MenuSnapshot,
}

impl TrayIcon {
    pub fn new(id: TrayIconId, attrs: TrayIconAttributes) -> crate::Result<Self> {
        let (parent_path, icon_path) = temp_icon_path(attrs.temp_dir_path.as_ref(), &id, 0)?;

        if let Some(icon) = attrs.icon {
            icon.inner.write_to_png(&icon_path)?;
        }

        let menu = attrs
            .menu
            .as_ref()
            .and_then(|menu| menu.as_menu().map(|menu| menu.items()))
            .map(|items| MenuSnapshot {
                items: snapshot_items(items),
            })
            .unwrap_or_default();

        let state = TrayState {
            title: attrs.title.clone().unwrap_or_default(),
            tooltip: attrs.tooltip.clone().unwrap_or_default(),
            icon_path: Some(icon_path.clone()),
            icon_theme_path: Some(parent_path),
            visible: true,
            menu,
        };

        let mut tray = Self {
            id,
            handle: None,
            state: Arc::new(Mutex::new(state)),
            path: icon_path,
            temp_dir_path: attrs.temp_dir_path,
            counter: 0,
            menu: attrs.menu,
        };

        tray.spawn()?;

        Ok(tray)
    }

    fn handler(&self) -> TrayHandler {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());

        TrayHandler {
            id: self.id.clone(),
            title: state.title.clone(),
            tooltip: state.tooltip.clone(),
            icon_path: state.icon_path.clone(),
            icon_theme_path: state.icon_theme_path.clone(),
            visible: state.visible,
            menu: state.menu.clone(),
        }
    }

    fn spawn(&mut self) -> crate::Result<()> {
        let handler = self.handler();

        let handle = async_io::block_on(handler.spawn()).map_err(|e| {
            crate::Error::OsError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        self.handle = Some(handle);

        Ok(())
    }

    /// Pushes the current state to the running tray.
    fn refresh(&self) {
        let Some(handle) = &self.handle else {
            return;
        };

        let next = self.handler();

        async_io::block_on(handle.update(move |tray: &mut TrayHandler| {
            tray.title = next.title.clone();
            tray.tooltip = next.tooltip.clone();
            tray.icon_path = next.icon_path.clone();
            tray.icon_theme_path = next.icon_theme_path.clone();
            tray.visible = next.visible;
            tray.menu = next.menu.clone();
        }));
    }

    pub fn set_icon(&mut self, icon: Option<Icon>) -> crate::Result<()> {
        let _ = std::fs::remove_file(&self.path);

        self.counter += 1;

        let (parent_path, icon_path) =
            temp_icon_path(self.temp_dir_path.as_ref(), &self.id, self.counter)?;

        if let Some(icon) = icon {
            icon.inner.write_to_png(&icon_path)?;
        }

        {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            state.icon_theme_path = Some(parent_path);
            state.icon_path = Some(icon_path.clone());
        }

        self.path = icon_path;
        self.refresh();

        Ok(())
    }

    pub fn set_menu(&mut self, menu: Option<Box<dyn crate::menu::ContextMenu>>) {
        {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            state.menu = menu
                .as_ref()
                .and_then(|menu| menu.as_menu().map(|menu| menu.items()))
                .map(|items| MenuSnapshot {
                    items: snapshot_items(items),
                })
                .unwrap_or_default();
        }

        self.menu = menu;
        self.refresh();
    }

    pub fn set_tooltip<S: AsRef<str>>(&mut self, tooltip: Option<S>) -> crate::Result<()> {
        {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            state.tooltip = tooltip
                .as_ref()
                .map(|t| t.as_ref().to_string())
                .unwrap_or_default();
        }

        self.refresh();

        Ok(())
    }

    pub fn set_title<S: AsRef<str>>(&mut self, title: Option<S>) {
        {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            state.title = title
                .as_ref()
                .map(|t| t.as_ref().to_string())
                .unwrap_or_default();
        }

        self.refresh();
    }

    pub fn set_visible(&mut self, visible: bool) -> crate::Result<()> {
        {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            state.visible = visible;
        }

        self.refresh();

        Ok(())
    }

    pub fn set_temp_dir_path<P: AsRef<Path>>(&mut self, path: Option<P>) {
        self.temp_dir_path = path.map(|p| p.as_ref().to_path_buf());
    }

    /// The protocol never reports where the host drew the icon.
    pub fn rect(&self) -> Option<crate::Rect> {
        None
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            async_io::block_on(handle.shutdown());
        }

        let _ = std::fs::remove_file(&self.path);
    }
}

/// Generates an icon path in one of the following dirs:
/// 1. If `temp_icon_dir` is `Some` use that.
/// 2. `$XDG_RUNTIME_DIR/tray-icon`
/// 3. `/tmp/tray-icon`
fn temp_icon_path(
    temp_icon_dir: Option<&PathBuf>,
    id: &TrayIconId,
    counter: u32,
) -> std::io::Result<(PathBuf, PathBuf)> {
    let parent_path = match temp_icon_dir.as_ref() {
        Some(path) => path.to_path_buf(),
        None => dirs::runtime_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("tray-icon"),
    };

    std::fs::create_dir_all(&parent_path)?;
    let icon_path = parent_path.join(format!("tray-icon-{}-{}.png", id.as_ref(), counter));
    Ok((parent_path, icon_path))
}
