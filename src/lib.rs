// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! tray-icon lets you create tray icons for desktop applications.
//!
//! # Example
//!
//! Create a tray icon without a menu.
//!
//! ```no_run
//! use tray_icon::{TrayIconBuilder, icon::Icon};
//!
//! # let icon = Icon::from_rgba(Vec::new(), 0, 0).unwrap();
//! let tray_icon = TrayIconBuilder::new()
//!     .with_tooltip("system-tray - tray icon library!")
//!     .with_icon(icon)
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Example
//!
//! Create a tray icon with a menu.
//!
//! ```no_run
//! use tray_icon::{TrayIconBuilder, menu::Menu, icon::Icon};
//!
//! # let icon = Icon::from_rgba(Vec::new(), 0, 0).unwrap();
//! let tray_menu = Menu::new();
//! let tray_icon = TrayIconBuilder::new()
//!     .with_menu(Box::new(tray_menu))
//!     .with_tooltip("system-tray - tray icon library!")
//!     .with_icon(icon)
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Processing tray events
//!
//! You can use [`tray_event_receiver`] to get a reference to the [`TrayEventReceiver`]
//! which you can use to listen to events when a click happens on the tray icon
//! ```no_run
//! use tray_icon::tray_event_receiver;
//!
//! if let Ok(event) = tray_event_receiver().try_recv() {
//!     println!("{:?}", event);
//! }
//! ```
//!
//! You can also listen for the menu events using [`menu_event_receiver`](crate::menu::menu_event_receiver) to get events for the tray context menu.
//!
//! ```no_run
//! use tray_icon::{tray_event_receiver, menu::menu_event_receiver};
//!
//! if let Ok(event) = tray_event_receiver().try_recv() {
//!     println!("tray event: {:?}", event);
//! }
//!
//! if let Ok(event) = menu_event_receiver().try_recv() {
//!     println!("menu event: {:?}", event);
//! }
//! ```

use std::path::{Path, PathBuf};

use counter::Counter;
use crossbeam_channel::{unbounded, Receiver, Sender};
use icon::Icon;
use once_cell::sync::Lazy;

mod counter;
mod error;
pub mod icon;
mod platform_impl;

pub use self::error::*;

/// Re-export of [muda](::muda) crate and used for tray context menu.
pub mod menu {
    pub use muda::*;
}

static COUNTER: Counter = Counter::new();

/// Describes a menu event emitted when a menu item is activated
#[derive(Debug)]
pub struct TrayEvent {
    /// Id of the tray icon which triggered this event
    pub id: u32,
    pub x: f64,
    pub y: f64,
    pub icon_rect: Rectangle,
    pub event: ClickEvent,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClickEvent {
    Left,
    Right,
    Double,
}

/// Describes a rectangle including position (x - y axis) and size.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Rectangle {
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
}

/// A reciever that could be used to listen to tray events.
pub type TrayEventReceiver = Receiver<TrayEvent>;

static TRAY_CHANNEL: Lazy<(Sender<TrayEvent>, TrayEventReceiver)> = Lazy::new(unbounded);

/// Gets a reference to the event channel's [TrayEventReceiver]
/// which can be used to listen for tray events.
pub fn tray_event_receiver<'a>() -> &'a TrayEventReceiver {
    &TRAY_CHANNEL.1
}

/// Attributes to use when creating a tray icon.
pub struct TrayIconAttributes {
    /// Tooltip for the tray icon
    ///
    /// ## Platform-specific:
    ///
    /// - **Linux:** Unsupported.
    pub tooltip: Option<String>,

    /// Tray menu
    pub menu: Option<Box<dyn menu::ContextMenu>>,

    /// Tray icon
    pub icon: Option<Icon>,

    /// Tray icon temp dir path. **Linux only**.
    pub temp_dir_path: Option<PathBuf>,

    /// Use the icon as a [template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc). **macOS only**.
    pub icon_is_template: bool,

    /// Whether to show the tray menu on left click or not, default is `true`. **macOS only**.
    pub menu_on_left_click: bool,
}

impl Default for TrayIconAttributes {
    fn default() -> Self {
        Self {
            tooltip: None,
            menu: None,
            icon: None,
            temp_dir_path: None,
            icon_is_template: false,
            menu_on_left_click: true,
        }
    }
}

#[derive(Default)]
pub struct TrayIconBuilder {
    attrs: TrayIconAttributes,
}

impl TrayIconBuilder {
    pub fn new() -> Self {
        Self {
            attrs: TrayIconAttributes::default(),
        }
    }

    /// ## Platform-specific:
    ///
    /// - **Linux**: once a menu is set it cannot be removed.
    pub fn with_menu(mut self, menu: Box<dyn menu::ContextMenu>) -> Self {
        self.attrs.menu = Some(menu);
        self
    }

    pub fn with_icon(mut self, icon: Icon) -> Self {
        self.attrs.icon = Some(icon);
        self
    }

    pub fn with_tooltip<S: AsRef<str>>(mut self, s: S) -> Self {
        self.attrs.tooltip = Some(s.as_ref().to_string());
        self
    }

    /// Set tray icon temp dir path. **Linux only**.
    pub fn with_temp_dir_path<P: AsRef<Path>>(mut self, s: P) -> Self {
        self.attrs.temp_dir_path = Some(s.as_ref().to_path_buf());
        self
    }

    /// Use the icon as a [template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc). **macOS only**.
    pub fn with_icon_as_template(mut self, is_template: bool) -> Self {
        self.attrs.icon_is_template = is_template;
        self
    }

    /// Whether to show the tray menu on left click or not, default is `true`. **macOS only**.
    pub fn with_menu_on_left_click(mut self, enable: bool) -> Self {
        self.attrs.menu_on_left_click = enable;
        self
    }

    pub fn build(self) -> Result<TrayIcon> {
        TrayIcon::new(self.attrs)
    }
}

pub struct TrayIcon {
    id: u32,
    tray: platform_impl::TrayIcon,
}

impl TrayIcon {
    pub fn new(attrs: TrayIconAttributes) -> Result<Self> {
        let id = COUNTER.next();
        Ok(Self {
            id,
            tray: platform_impl::TrayIcon::new(id, attrs)?,
        })
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    /// Set new tray icon. If `None` is provided, it will hide the icon.
    pub fn set_icon(&mut self, icon: Option<Icon>) -> Result<()> {
        self.tray.set_icon(icon)
    }

    /// Set new tray menu.
    ///
    /// ## Platform-specific:
    ///
    /// - **Linux**: once a menu is set it cannot be removed so `None` has no effect
    pub fn set_menu(&mut self, menu: Option<Box<dyn menu::ContextMenu>>) {
        self.tray.set_menu(menu)
    }

    /// Sets the tooltip for this tray icon.
    ///
    /// ## Platform-specific:
    ///
    /// - **Linux:** Unsupported
    pub fn set_tooltip<S: AsRef<str>>(&mut self, tooltip: Option<S>) -> Result<()> {
        self.tray.set_tooltip(tooltip)
    }

    /// Sets the tray icon temp dir path. **Linux only**.
    pub fn set_temp_dir_path<P: AsRef<Path>>(&mut self, path: Option<P>) {
        #[cfg(target_os = "linux")]
        self.tray.set_temp_dir_path(path);
        #[cfg(not(target_os = "linux"))]
        let _ = path;
    }

    /// Set the current icon as a [template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc). **macOS only**.
    pub fn set_icon_as_template(&mut self, is_template: bool) {
        #[cfg(target_os = "macos")]
        self.tray.set_icon_as_template(is_template);
        #[cfg(not(target_os = "macos"))]
        let _ = is_template;
    }

    /// Disable or enable showing the tray menu on left click. **macOS only**.
    pub fn set_show_menu_on_left_click(&mut self, enable: bool) {
        #[cfg(target_os = "macos")]
        self.tray.set_show_menu_on_left_click(enable);
        #[cfg(not(target_os = "macos"))]
        let _ = enable;
    }
}
