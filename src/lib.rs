use counter::Counter;
use crossbeam_channel::{unbounded, Receiver, Sender};
use icon::Icon;
use once_cell::sync::Lazy;

mod counter;
pub mod icon;
mod platform_impl;

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
    pub x: f32,
    pub y: f32,
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
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
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
#[derive(Default)]
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

    pub fn build(self) -> Result<TrayIcon, ()> {
        TrayIcon::new(self.attrs)
    }
}

pub struct TrayIcon {
    id: u32,
    tray: platform_impl::TrayIcon,
}

impl TrayIcon {
    pub fn new(attrs: TrayIconAttributes) -> Result<Self, ()> {
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
    pub fn set_icon(&mut self, icon: Option<Icon>) {
        self.tray.set_icon(icon)
    }

    /// Set new tray menu.
    pub fn set_menu(&mut self, menu: Option<Box<dyn menu::ContextMenu>>) {
        self.tray.set_menu(menu)
    }

    /// Sets the tooltip for this tray icon.
    ///
    /// ## Platform-specific:
    ///
    /// - **Linux:** Unsupported
    pub fn set_tooltip<S: AsRef<str>>(&mut self, tooltip: Option<S>) {
        self.tray.set_tooltip(tooltip);
    }
}
