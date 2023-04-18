// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod icon;
use std::sync::Once;

use cocoa::{
    appkit::{
        NSButton, NSEventMask, NSEventModifierFlags, NSEventType, NSImage, NSStatusBar,
        NSStatusItem, NSVariableStatusItemLength, NSWindow,
    },
    base::{id, nil},
    foundation::{NSData, NSPoint, NSRect, NSSize, NSString},
};
use core_graphics::display::CGDisplay;
pub(crate) use icon::PlatformIcon;
use objc::{
    class,
    declare::ClassDecl,
    msg_send,
    runtime::{Class, Object, Protocol, Sel},
    sel, sel_impl,
};

use crate::{icon::Icon, menu, ClickEvent, Rectangle, TrayEvent, TrayIconAttributes};

const TRAY_ID: &str = "id";
const TRAY_MENU: &str = "menu";
const TRAY_MENU_ON_LEFT_CLICK: &str = "menu_on_left_click";
const TRAY_STATUS_BAR: &str = "status_bar";

pub struct TrayIcon {
    ns_status_bar: Option<id>,
    tray_target: Option<id>,
    id: u32,
    attrs: TrayIconAttributes,
}

impl TrayIcon {
    pub fn new(id: u32, attrs: TrayIconAttributes) -> crate::Result<Self> {
        let (ns_status_bar, tray_target) = Self::create(id, &attrs)?;

        let tray_icon = Self {
            ns_status_bar: Some(ns_status_bar),
            tray_target: Some(tray_target),
            id,
            attrs,
        };

        Ok(tray_icon)
    }

    fn create(id: u32, attrs: &TrayIconAttributes) -> crate::Result<(id, id)> {
        let ns_status_bar = unsafe {
            let ns_status_bar =
                NSStatusBar::systemStatusBar(nil).statusItemWithLength_(NSVariableStatusItemLength);
            let _: () = msg_send![ns_status_bar, retain];
            ns_status_bar
        };

        set_icon_for_ns_status_bar_button(
            ns_status_bar,
            attrs.icon.clone(),
            attrs.icon_is_template,
        );

        // attach click event to our button
        let tray_target: id = unsafe {
            let button = ns_status_bar.button();
            let target: id = msg_send![make_tray_class(), alloc];
            let tray_target: id = msg_send![target, init];
            let _: () = msg_send![target, retain];

            (*tray_target).set_ivar(TRAY_ID, id);
            (*tray_target).set_ivar(TRAY_STATUS_BAR, ns_status_bar);
            (*tray_target).set_ivar(TRAY_MENU, nil);
            (*tray_target).set_ivar(TRAY_MENU_ON_LEFT_CLICK, attrs.menu_on_left_click);

            let _: () = msg_send![button, setAction: sel!(click:)];
            let _: () = msg_send![button, setTarget: tray_target];
            let _: () = msg_send![
                button,
                sendActionOn: NSEventMask::NSLeftMouseDownMask
                    | NSEventMask::NSRightMouseDownMask
                    | NSEventMask::NSKeyDownMask
            ];

            if let Some(menu) = &attrs.menu {
                // We set the tray menu to tray_target instead of status bar
                // Because setting directly to status bar will overwrite the event callback of the button
                // See `make_tray_class` for more information.
                let menu: id = menu.ns_menu() as _;
                (*tray_target).set_ivar("menu", menu);
                let () = msg_send![menu, setDelegate: tray_target];
            }

            tray_target
        };

        Self::set_tooltip_inner(ns_status_bar, attrs.tooltip.clone())?;
        Self::set_title_inner(ns_status_bar, attrs.title.clone());

        Ok((ns_status_bar, tray_target))
    }

    fn remove(&mut self) {
        if let (Some(ns_status_bar), Some(tray_target)) = (&self.ns_status_bar, &self.tray_target) {
            unsafe {
                NSStatusBar::systemStatusBar(nil).removeStatusItem_(*ns_status_bar);
                let _: () = msg_send![*ns_status_bar, release];
                let _: () = msg_send![*tray_target, release];
            }
        }
        self.ns_status_bar = None;
        self.tray_target = None;
    }

    pub fn set_icon(&mut self, icon: Option<Icon>) -> crate::Result<()> {
        if let Some(ns_status_bar) = self.ns_status_bar {
            set_icon_for_ns_status_bar_button(ns_status_bar, icon.clone(), false);
        }
        self.attrs.icon = icon;
        Ok(())
    }

    pub fn set_menu(&mut self, menu: Option<Box<dyn menu::ContextMenu>>) {
        if let Some(tray_target) = self.tray_target {
            unsafe {
                let menu = menu.as_ref().map(|m| m.ns_menu() as _).unwrap_or(nil);
                (*tray_target).set_ivar(TRAY_MENU, menu);
            }
        }
        self.attrs.menu = menu;
    }

    pub fn set_tooltip<S: AsRef<str>>(&mut self, tooltip: Option<S>) -> crate::Result<()> {
        let tooltip = tooltip.map(|s| s.as_ref().to_string());
        if let Some(ns_status_bar) = self.ns_status_bar {
            Self::set_tooltip_inner(ns_status_bar, tooltip.clone())?;
        }
        self.attrs.tooltip = tooltip;
        Ok(())
    }

    fn set_tooltip_inner<S: AsRef<str>>(
        ns_status_bar: id,
        tooltip: Option<S>,
    ) -> crate::Result<()> {
        unsafe {
            let tooltip = match tooltip {
                Some(tooltip) => NSString::alloc(nil).init_str(tooltip.as_ref()),
                None => nil,
            };
            let _: () = msg_send![ns_status_bar.button(), setToolTip: tooltip];
        }
        Ok(())
    }

    pub fn set_title<S: AsRef<str>>(&mut self, title: Option<S>) {
        let title = title.map(|s| s.as_ref().to_string());
        if let Some(ns_status_bar) = self.ns_status_bar {
            Self::set_title_inner(ns_status_bar, title.clone());
        }
        self.attrs.title = title;
    }

    fn set_title_inner<S: AsRef<str>>(ns_status_bar: id, title: Option<S>) {
        unsafe {
            let title = match title {
                Some(title) => NSString::alloc(nil).init_str(title.as_ref()),
                None => nil,
            };
            let _: () = msg_send![ns_status_bar.button(), setTitle: title];
        }
    }

    pub fn set_visible(&mut self, visible: bool) -> crate::Result<()> {
        if visible {
            if self.ns_status_bar.is_none() {
                let (ns_status_bar, tray_target) = Self::create(self.id, &self.attrs)?;
                self.ns_status_bar = Some(ns_status_bar);
                self.tray_target = Some(tray_target);
            }
        } else {
            self.remove();
        }

        Ok(())
    }

    pub fn set_icon_as_template(&mut self, is_template: bool) {
        if let Some(ns_status_bar) = self.ns_status_bar {
            unsafe {
                let button = ns_status_bar.button();
                let nsimage: id = msg_send![button, image];
                let _: () = msg_send![nsimage, setTemplate: is_template as i8];
            }
        }
        self.attrs.icon_is_template = is_template;
    }

    pub fn set_show_menu_on_left_click(&mut self, enable: bool) {
        if let Some(tray_target) = self.tray_target {
            unsafe {
                (*tray_target).set_ivar(TRAY_MENU_ON_LEFT_CLICK, enable);
            }
        }
        self.attrs.menu_on_left_click = enable;
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        self.remove()
    }
}

fn set_icon_for_ns_status_bar_button(
    ns_status_bar: id,
    icon: Option<Icon>,
    icon_is_template: bool,
) {
    let button = unsafe { ns_status_bar.button() };

    if let Some(icon) = icon {
        // The image is to the right of the title https://developer.apple.com/documentation/appkit/nscellimageposition/nsimageleft
        const NSIMAGE_LEFT: i32 = 2;

        let png_icon = icon.inner.to_png();

        let (width, height) = icon.inner.get_size();

        let icon_height: f64 = 18.0;
        let icon_width: f64 = (width as f64) / (height as f64 / icon_height);

        unsafe {
            // build our icon
            let nsdata = NSData::dataWithBytes_length_(
                nil,
                png_icon.as_ptr() as *const std::os::raw::c_void,
                png_icon.len() as u64,
            );

            let nsimage = NSImage::initWithData_(NSImage::alloc(nil), nsdata);
            let new_size = NSSize::new(icon_width, icon_height);

            button.setImage_(nsimage);
            let _: () = msg_send![nsimage, setSize: new_size];
            let _: () = msg_send![button, setImagePosition: NSIMAGE_LEFT];
            let _: () = msg_send![nsimage, setTemplate: icon_is_template as i8];
        }
    } else {
        unsafe { button.setImage_(nil) };
    }
}

/// Create a `TrayHandler` Class that handle button click event and also menu opening and closing.
///
/// We set the tray menu to tray_target instead of status bar, because setting directly to status bar
/// will overwrite the event callback of the button. When `perform_tray_click` called, it will set
/// the menu to status bar in the end. And when the menu is closed `menu_did_close` will set it to
/// nil again.
fn make_tray_class() -> *const Class {
    static mut TRAY_CLASS: *const Class = 0 as *const Class;
    static INIT: Once = Once::new();

    INIT.call_once(|| unsafe {
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("TaoTrayHandler", superclass).unwrap();
        decl.add_ivar::<id>(TRAY_STATUS_BAR);
        decl.add_ivar::<id>(TRAY_MENU);
        decl.add_ivar::<bool>(TRAY_MENU_ON_LEFT_CLICK);
        decl.add_ivar::<u32>(TRAY_ID);
        decl.add_method(
            sel!(click:),
            perform_tray_click as extern "C" fn(&mut Object, _, id),
        );

        let delegate = Protocol::get("NSMenuDelegate").unwrap();
        decl.add_protocol(delegate);
        decl.add_method(
            sel!(menuDidClose:),
            menu_did_close as extern "C" fn(&mut Object, _, id),
        );

        TRAY_CLASS = decl.register();
    });

    unsafe { TRAY_CLASS }
}

/// This will fire for an NSButton callback.
extern "C" fn perform_tray_click(this: &mut Object, _: Sel, button: id) {
    unsafe {
        let id = *this.get_ivar::<u32>(TRAY_ID);
        let app: id = msg_send![class!(NSApplication), sharedApplication];
        let current_event: id = msg_send![app, currentEvent];

        // icon position & size
        let window: id = msg_send![current_event, window];
        let frame = NSWindow::frame(window);
        let scale_factor = NSWindow::backingScaleFactor(window);
        let (tray_x, tray_y) = (
            frame.origin.x * scale_factor,
            bottom_left_to_top_left_for_tray(frame) * scale_factor,
        );

        let (tray_width, tray_height) = (
            frame.size.width * scale_factor,
            frame.size.height * scale_factor,
        );

        // cursor position
        let mouse_location: NSPoint = msg_send![class!(NSEvent), mouseLocation];
        // what type of click?
        let event_mask: NSEventType = msg_send![current_event, type];
        // grab the modifier flag, to make sure the ctrl + left click = right click
        let key_code: NSEventModifierFlags = msg_send![current_event, modifierFlags];

        let click_type = match event_mask {
            // left click + control key
            NSEventType::NSLeftMouseDown
                if key_code.contains(NSEventModifierFlags::NSControlKeyMask) =>
            {
                Some(ClickEvent::Right)
            }
            NSEventType::NSLeftMouseDown => Some(ClickEvent::Left),
            NSEventType::NSRightMouseDown => Some(ClickEvent::Right),
            _ => None,
        };

        if let Some(click_event) = click_type {
            let event = TrayEvent {
                id,
                x: mouse_location.x,
                y: bottom_left_to_top_left_for_cursor(mouse_location),
                icon_rect: Rectangle {
                    left: tray_x,
                    right: tray_x + tray_width,
                    top: tray_y,
                    bottom: tray_y + tray_height,
                },
                event: click_event,
            };

            TrayEvent::send(event);

            let menu = this.get_ivar::<id>(TRAY_MENU);
            if *menu != nil {
                let menu_on_left_click = *this.get_ivar::<bool>(TRAY_MENU_ON_LEFT_CLICK);
                if click_event == ClickEvent::Right
                    || (menu_on_left_click && click_event == ClickEvent::Left)
                {
                    let status_bar = this.get_ivar::<id>(TRAY_STATUS_BAR);
                    status_bar.setMenu_(*menu);
                    let () = msg_send![button, performClick: nil];
                }
            }
        }
    }
}

// Set the menu of the status bar to nil, so it won't overwrite the button events.
extern "C" fn menu_did_close(this: &mut Object, _: Sel, _menu: id) {
    unsafe {
        let status_bar = this.get_ivar::<id>(TRAY_STATUS_BAR);
        status_bar.setMenu_(nil);
    }
}

/// Get the icon Y-axis correctly aligned with tao based on the tray icon `NSRect`.
/// Available only with the `tray` feature flag.
pub fn bottom_left_to_top_left_for_tray(rect: NSRect) -> f64 {
    CGDisplay::main().pixels_high() as f64 - rect.origin.y
}

/// Get the cursor Y-axis correctly aligned with tao when we click on the tray icon.
/// Available only with the `tray` feature flag.
pub fn bottom_left_to_top_left_for_cursor(point: NSPoint) -> f64 {
    CGDisplay::main().pixels_high() as f64 - point.y
}
