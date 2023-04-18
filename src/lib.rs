// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(clippy::uninlined_format_args)]

//! tray-icon lets you create tray icons for desktop applications.
//!
//! # Platforms supported:
//!
//! - Windows
//! - macOS
//! - Linux (gtk Only)
//!
//! # Platform-specific notes:
//!
//! - On Windows and Linux, an event loop must be running on the thread, on Windows, a win32 event loop and on Linux, a gtk event loop. It doesn't need to be the main thread but you have to create the tray icon on the same thread as the event loop.
//! - On macOS, an event loop must be running on the main thread so you also need to create the tray icon on the main thread.
//!
//! # Dependencies (Linux Only)
//!
//! On Linux, `gtk` and `libappindicator` or `libayatnat-appindicator` are used to create the tray icon, so make sure to install them on your system.
//!
//! #### Arch Linux / Manjaro:
//!
//! ```sh
//! pacman -S gtk3 libappindicator-gtk3 #or libayatana-appindicator
//! ```
//!
//! #### Debian / Ubuntu:
//!
//! ```sh
//! sudo apt install libgtk-3-dev libappindicator3-dev #or libayatana-appindicator3-dev
//! ```
//! if you use `tray_icon::menu` module, make sure to checkout <https://github.com/tauri-apps/muda#dependencies>
//!
//! # Examples
//!
//! #### Create a tray icon without a menu.
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
//! #### Create a tray icon with a menu.
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
//! You can use [`TrayEvent::receiver`] to get a reference to the [`TrayEventReceiver`]
//! which you can use to listen to events when a click happens on the tray icon
//! ```no_run
//! use tray_icon::TrayEvent;
//!
//! if let Ok(event) = TrayEvent::receiver().try_recv() {
//!     println!("{:?}", event);
//! }
//! ```
//!
//! You can also listen for the menu events using [`TrayEvent::receiver`] to get events for the tray context menu.
//!
//! ```no_run
//! use tray_icon::{TrayEvent, menu::MenuEvent};
//!
//! if let Ok(event) = TrayEvent::receiver().try_recv() {
//!     println!("tray event: {:?}", event);
//! }
//!
//! if let Ok(event) = MenuEvent::receiver().try_recv() {
//!     println!("menu event: {:?}", event);
//! }
//! ```

use std::path::{Path, PathBuf};

use counter::Counter;
use crossbeam_channel::{unbounded, Receiver, Sender};
use icon::Icon;
use once_cell::sync::{Lazy, OnceCell};

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

/// Attributes to use when creating a tray icon.
pub struct TrayIconAttributes {
    /// Tray icon tooltip
    ///
    /// ## Platform-specific:
    ///
    /// - **Linux:** Unsupported.
    pub tooltip: Option<String>,

    /// Tray menu
    ///
    /// ## Platform-specific:
    ///
    /// - **Linux**: once a menu is set, it cannot be removed.
    pub menu: Option<Box<dyn menu::ContextMenu>>,

    /// Tray icon
    ///
    /// ## Platform-specific:
    ///
    /// - **Linux:** Sometimes the icon won't be visible unless a menu is set.
    ///     Setting an empty [`Menu`](crate::menu::Menu) is enough.
    pub icon: Option<Icon>,

    /// Tray icon temp dir path. **Linux only**.
    pub temp_dir_path: Option<PathBuf>,

    /// Use the icon as a [template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc). **macOS only**.
    pub icon_is_template: bool,

    /// Whether to show the tray menu on left click or not, default is `true`. **macOS only**.
    pub menu_on_left_click: bool,

    /// Tray icon title.
    ///
    /// ## Platform-specific
    ///
    /// - **Linux:** The title will not be shown unless there is an icon
    /// as well.  The title is useful for numerical and other frequently
    /// updated information.  In general, it shouldn't be shown unless a
    /// user requests it as it can take up a significant amount of space
    /// on the user's panel.  This may not be shown in all visualizations.
    /// - **Windows:** Unsupported.
    pub title: Option<String>,
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
            title: None,
        }
    }
}

/// [`TrayIcon`] builder struct and associated methods.
#[derive(Default)]
pub struct TrayIconBuilder {
    attrs: TrayIconAttributes,
}

impl TrayIconBuilder {
    /// Creates a new [`TrayIconBuilder`] with default [`TrayIconAttributes`].
    pub fn new() -> Self {
        Self {
            attrs: TrayIconAttributes::default(),
        }
    }

    /// Set the a menu for this tray icon.
    ///
    /// ## Platform-specific:
    ///
    /// - **Linux**: once a menu is set, it cannot be removed or replaced but you can change its content.
    pub fn with_menu(mut self, menu: Box<dyn menu::ContextMenu>) -> Self {
        self.attrs.menu = Some(menu);
        self
    }

    /// Set an icon for this tray icon.
    ///
    /// ## Platform-specific:
    ///
    /// - **Linux:** Sometimes the icon won't be visible unless a menu is set.
    /// Setting an empty [`Menu`](crate::menu::Menu) is enough.
    pub fn with_icon(mut self, icon: Icon) -> Self {
        self.attrs.icon = Some(icon);
        self
    }

    /// Set a tooltip for this tray icon.
    ///
    /// ## Platform-specific:
    ///
    /// - **Linux:** Unsupported.
    pub fn with_tooltip<S: AsRef<str>>(mut self, s: S) -> Self {
        self.attrs.tooltip = Some(s.as_ref().to_string());
        self
    }

    /// Set the tray icon title.
    ///
    /// ## Platform-specific
    ///
    /// - **Linux:** The title will not be shown unless there is an icon
    /// as well.  The title is useful for numerical and other frequently
    /// updated information.  In general, it shouldn't be shown unless a
    /// user requests it as it can take up a significant amount of space
    /// on the user's panel.  This may not be shown in all visualizations.
    /// - **Windows:** Unsupported.
    pub fn with_title<S: AsRef<str>>(mut self, title: S) -> Self {
        self.attrs.title.replace(title.as_ref().to_string());
        self
    }

    /// Set tray icon temp dir path. **Linux only**.
    ///
    /// On Linux, we need to write the icon to the disk and usually it will
    /// be `$XDG_RUNTIME_DIR/tray-icon` or `$TEMP/tray-icon`.
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

    /// Builds and adds a new [`TrayIcon`] to the system tray.
    pub fn build(self) -> Result<TrayIcon> {
        TrayIcon::new(self.attrs)
    }
}

/// Tray icon struct and associated methods.
pub struct TrayIcon {
    id: u32,
    tray: platform_impl::TrayIcon,
}

impl TrayIcon {
    /// Builds and adds a new tray icon to the system tray.
    ///
    /// ## Platform-specific:
    ///
    /// - **Linux:** Sometimes the icon won't be visible unless a menu is set.
    /// Setting an empty [`Menu`](crate::menu::Menu) is enough.
    pub fn new(attrs: TrayIconAttributes) -> Result<Self> {
        let id = COUNTER.next();
        Ok(Self {
            id,
            tray: platform_impl::TrayIcon::new(id, attrs)?,
        })
    }

    /// Returns the id associated with this tray icon.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Set new tray icon. If `None` is provided, it will remove the icon.
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

    /// Sets the tooltip for this tray icon.
    ///
    /// ## Platform-specific:
    ///
    /// - **Linux:** The title will not be shown unless there is an icon
    /// as well.  The title is useful for numerical and other frequently
    /// updated information.  In general, it shouldn't be shown unless a
    /// user requests it as it can take up a significant amount of space
    /// on the user's panel.  This may not be shown in all visualizations.
    /// - **Windows:** Unsupported
    pub fn set_title<S: AsRef<str>>(&mut self, title: Option<S>) {
        self.tray.set_title(title)
    }

    /// Show or hide this tray icon
    pub fn set_visible(&mut self, visible: bool) -> Result<()> {
        self.tray.set_visible(visible)
    }

    /// Sets the tray icon temp dir path. **Linux only**.
    ///
    /// On Linux, we need to write the icon to the disk and usually it will
    /// be `$XDG_RUNTIME_DIR/tray-icon` or `$TEMP/tray-icon`.
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
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

/// A reciever that could be used to listen to tray events.
pub type TrayEventReceiver = Receiver<TrayEvent>;
type TrayEventHandler = Box<dyn Fn(TrayEvent) + Send + Sync + 'static>;

static TRAY_CHANNEL: Lazy<(Sender<TrayEvent>, TrayEventReceiver)> = Lazy::new(unbounded);
static TRAY_EVENT_HANDLER: OnceCell<Option<TrayEventHandler>> = OnceCell::new();

impl TrayEvent {
    /// Gets a reference to the event channel's [`TrayEventReceiver`]
    /// which can be used to listen for tray events.
    ///
    /// ## Note
    ///
    /// This will not receive any events if [`TrayEvent::set_event_handler`] has been called with a `Some` value.
    pub fn receiver<'a>() -> &'a TrayEventReceiver {
        &TRAY_CHANNEL.1
    }

    /// Set a handler to be called for new events. Useful for implementing custom event sender.
    ///
    /// ## Note
    ///
    /// Calling this function with a `Some` value,
    /// will not send new events to the channel associated with [`TrayEvent::receiver`]
    pub fn set_event_handler<F: Fn(TrayEvent) + Send + Sync + 'static>(f: Option<F>) {
        if let Some(f) = f {
            let _ = TRAY_EVENT_HANDLER.set(Some(Box::new(f)));
        } else {
            let _ = TRAY_EVENT_HANDLER.set(None);
        }
    }

    #[allow(unused)]
    pub(crate) fn send(event: TrayEvent) {
        if let Some(handler) = TRAY_EVENT_HANDLER.get_or_init(|| None) {
            handler(event);
        } else {
            let _ = TRAY_CHANNEL.0.send(event);
        }
    }
}
