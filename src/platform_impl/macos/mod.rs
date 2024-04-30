// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod icon;
use std::sync::Once;

use cocoa::{
    appkit::{
        NSButton, NSEvent, NSImage, NSStatusBar, NSStatusItem, NSVariableStatusItemLength, NSWindow,
    },
    base::{id, nil},
    foundation::{NSData, NSInteger, NSPoint, NSRect, NSSize, NSString},
};
use core_graphics::display::CGDisplay;
pub(crate) use icon::PlatformIcon;
use objc::{
    class,
    declare::ClassDecl,
    msg_send,
    runtime::{Class, Object, Sel, NO, YES},
    sel, sel_impl,
};

use crate::{icon::Icon, menu, ClickType, Rect, TrayIconAttributes, TrayIconEvent, TrayIconId};

const TRAY_ID: &str = "id";
const TRAY_STATUS_ITEM: &str = "status_item";
const TRAY_MENU: &str = "menu";
const TRAY_MENU_ON_LEFT_CLICK: &str = "menu_on_left_click";

pub struct TrayIcon {
    ns_status_item: Option<id>,
    tray_target: Option<id>,
    id: TrayIconId,
    attrs: TrayIconAttributes,
}

impl TrayIcon {
    pub fn new(id: TrayIconId, attrs: TrayIconAttributes) -> crate::Result<Self> {
        let (ns_status_item, tray_target) = Self::create(&id, &attrs)?;

        let tray_icon = Self {
            ns_status_item: Some(ns_status_item),
            tray_target: Some(tray_target),
            id,
            attrs,
        };

        Ok(tray_icon)
    }

    fn create(id: &TrayIconId, attrs: &TrayIconAttributes) -> crate::Result<(id, id)> {
        let ns_status_item = unsafe {
            let ns_status_item =
                NSStatusBar::systemStatusBar(nil).statusItemWithLength_(NSVariableStatusItemLength);
            let _: () = msg_send![ns_status_item, retain];
            ns_status_item
        };

        set_icon_for_ns_status_item_button(
            ns_status_item,
            attrs.icon.clone(),
            attrs.icon_is_template,
        )?;

        if let Some(menu) = &attrs.menu {
            unsafe {
                ns_status_item.setMenu_(menu.ns_menu() as _);
            }
        }

        Self::set_tooltip_inner(ns_status_item, attrs.tooltip.clone())?;
        Self::set_title_inner(ns_status_item, attrs.title.clone());

        let tray_target = unsafe {
            let button = ns_status_item.button();

            let frame: NSRect = msg_send![button, frame];

            let target: id = msg_send![make_tray_target_class(), alloc];
            let tray_target: id = msg_send![target, initWithFrame: frame];
            let _: () = msg_send![tray_target, retain];
            let _: () = msg_send![tray_target, setWantsLayer: YES];

            let id = NSString::alloc(nil).init_str(&id.0);

            (*tray_target).set_ivar(TRAY_ID, id);
            (*tray_target).set_ivar(TRAY_STATUS_ITEM, ns_status_item);
            (*tray_target).set_ivar(TRAY_MENU_ON_LEFT_CLICK, attrs.menu_on_left_click);
            if let Some(menu) = &attrs.menu {
                (*tray_target).set_ivar::<id>(TRAY_MENU, menu.ns_menu() as _);
            }

            let _: () = msg_send![button, addSubview: tray_target];

            tray_target
        };

        Ok((ns_status_item, tray_target))
    }

    fn remove(&mut self) {
        if let (Some(ns_status_item), Some(tray_target)) = (&self.ns_status_item, &self.tray_target)
        {
            unsafe {
                NSStatusBar::systemStatusBar(nil).removeStatusItem_(*ns_status_item);
                let _: () = msg_send![*tray_target, removeFromSuperview];
                let _: () = msg_send![*ns_status_item, release];
                let _: () = msg_send![*tray_target, release];
            }
        }

        self.ns_status_item = None;
        self.tray_target = None;
    }

    pub fn set_icon(&mut self, icon: Option<Icon>) -> crate::Result<()> {
        if let (Some(ns_status_item), Some(tray_target)) = (self.ns_status_item, self.tray_target) {
            set_icon_for_ns_status_item_button(ns_status_item, icon.clone(), false)?;
            unsafe {
                let _: () = msg_send![tray_target, updateDimensions];
            }
        }
        self.attrs.icon = icon;
        Ok(())
    }

    pub fn set_menu(&mut self, menu: Option<Box<dyn menu::ContextMenu>>) {
        if let (Some(ns_status_item), Some(tray_target)) = (self.ns_status_item, self.tray_target) {
            unsafe {
                let menu = menu.as_ref().map(|m| m.ns_menu() as _).unwrap_or(nil);
                (*tray_target).set_ivar(TRAY_MENU, menu);
                ns_status_item.setMenu_(menu);
                let () = msg_send![menu, setDelegate: ns_status_item];
            }
        }
        self.attrs.menu = menu;
    }

    pub fn set_tooltip<S: AsRef<str>>(&mut self, tooltip: Option<S>) -> crate::Result<()> {
        let tooltip = tooltip.map(|s| s.as_ref().to_string());
        if let (Some(ns_status_item), Some(tray_target)) = (self.ns_status_item, self.tray_target) {
            Self::set_tooltip_inner(ns_status_item, tooltip.clone())?;
            unsafe {
                let _: () = msg_send![tray_target, updateDimensions];
            }
        }
        self.attrs.tooltip = tooltip;
        Ok(())
    }

    fn set_tooltip_inner<S: AsRef<str>>(
        ns_status_item: id,
        tooltip: Option<S>,
    ) -> crate::Result<()> {
        unsafe {
            let tooltip = match tooltip {
                Some(tooltip) => NSString::alloc(nil).init_str(tooltip.as_ref()),
                None => nil,
            };
            let _: () = msg_send![ns_status_item.button(), setToolTip: tooltip];
        }
        Ok(())
    }

    pub fn set_title<S: AsRef<str>>(&mut self, title: Option<S>) {
        let title = title.map(|s| s.as_ref().to_string());
        if let (Some(ns_status_item), Some(tray_target)) = (self.ns_status_item, self.tray_target) {
            Self::set_title_inner(ns_status_item, title.clone());
            unsafe {
                let _: () = msg_send![tray_target, updateDimensions];
            }
        }
        self.attrs.title = title;
    }

    fn set_title_inner<S: AsRef<str>>(ns_status_item: id, title: Option<S>) {
        unsafe {
            let title = match title {
                Some(title) => NSString::alloc(nil).init_str(title.as_ref()),
                None => nil,
            };
            let _: () = msg_send![ns_status_item.button(), setTitle: title];
        }
    }

    pub fn set_visible(&mut self, visible: bool) -> crate::Result<()> {
        if visible {
            if self.ns_status_item.is_none() {
                let (ns_status_item, tray_target) = Self::create(&self.id, &self.attrs)?;
                self.ns_status_item = Some(ns_status_item);
                self.tray_target = Some(tray_target);
            }
        } else {
            self.remove();
        }

        Ok(())
    }

    pub fn set_icon_as_template(&mut self, is_template: bool) {
        if let Some(ns_status_item) = self.ns_status_item {
            unsafe {
                let button = ns_status_item.button();
                let nsimage: id = msg_send![button, image];
                let _: () = msg_send![nsimage, setTemplate: is_template as i8];
                button.setImage_(nsimage);
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

    pub fn rect(&self) -> Option<Rect> {
        let ns_status_item = self.ns_status_item?;
        unsafe {
            let button = ns_status_item.button();
            let window: id = button.window();
            if window.is_null() {
                None
            } else {
                Some(get_tray_rect(window))
            }
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        self.remove()
    }
}

fn set_icon_for_ns_status_item_button(
    ns_status_item: id,
    icon: Option<Icon>,
    icon_is_template: bool,
) -> crate::Result<()> {
    let button = unsafe { ns_status_item.button() };

    if let Some(icon) = icon {
        // The image is to the right of the title https://developer.apple.com/documentation/appkit/nscellimageposition/nsimageleft
        const NSIMAGE_LEFT: i32 = 2;

        let png_icon = icon.inner.to_png()?;

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

    Ok(())
}

/// Create a `TaoTrayTarget` Class that handle events.
fn make_tray_target_class() -> *const Class {
    static mut TRAY_CLASS: *const Class = 0 as *const Class;
    static INIT: Once = Once::new();

    INIT.call_once(|| unsafe {
        let superclass = class!(NSView);
        let mut decl =
            ClassDecl::new("TaoTrayTarget", superclass).expect("TaoTrayTarget already registered?");

        decl.add_ivar::<id>(TRAY_ID);
        decl.add_ivar::<id>(TRAY_MENU);
        decl.add_ivar::<id>(TRAY_STATUS_ITEM);
        decl.add_ivar::<bool>(TRAY_MENU_ON_LEFT_CLICK);

        decl.add_method(sel!(dealloc), dealloc as extern "C" fn(&mut Object, _));

        decl.add_method(
            sel!(mouseDown:),
            on_mouse_down as extern "C" fn(&mut Object, _, id),
        );
        decl.add_method(
            sel!(mouseUp:),
            on_mouse_up as extern "C" fn(&mut Object, _, id),
        );
        decl.add_method(
            sel!(rightMouseDown:),
            on_right_mouse_down as extern "C" fn(&mut Object, _, id),
        );

        decl.add_method(
            sel!(updateDimensions),
            update_dimensions as extern "C" fn(&mut Object, _),
        );

        extern "C" fn dealloc(this: &mut Object, _: Sel) {
            unsafe {
                this.set_ivar(TRAY_MENU, nil);
                this.set_ivar(TRAY_STATUS_ITEM, nil);

                let _: () = msg_send![super(this, class!(NSView)), dealloc];
            }
        }

        extern "C" fn on_right_mouse_down(this: &mut Object, _: Sel, event: id) {
            unsafe {
                on_tray_click(this, event, ClickType::Right);
            }
        }

        extern "C" fn on_mouse_up(this: &mut Object, _: Sel, _event: id) {
            unsafe {
                let ns_status_item = this.get_ivar::<id>(TRAY_STATUS_ITEM);
                let button: id = msg_send![*ns_status_item, button];
                let _: () = msg_send![button, highlight: NO];
            }
        }

        extern "C" fn on_mouse_down(this: &mut Object, _: Sel, event: id) {
            unsafe {
                on_tray_click(this, event, ClickType::Left);
            }
        }

        extern "C" fn update_dimensions(this: &mut Object, _: Sel) {
            unsafe {
                let ns_status_item = this.get_ivar::<id>(TRAY_STATUS_ITEM);
                let button: id = msg_send![*ns_status_item, button];

                let frame: NSRect = msg_send![button, frame];
                let _: () = msg_send![this, setFrame: frame];
            }
        }

        unsafe fn on_tray_click(this: &mut Object, event: id, click_event: ClickType) {
            const UTF8_ENCODING: usize = 4;

            let id_ns_str = *this.get_ivar::<id>(TRAY_ID);
            let bytes: *const std::ffi::c_char = msg_send![id_ns_str, UTF8String];
            let len = msg_send![id_ns_str, lengthOfBytesUsingEncoding: UTF8_ENCODING];
            let bytes = std::slice::from_raw_parts(bytes as *const u8, len);
            let id_str = std::str::from_utf8_unchecked(bytes);

            // icon position & size
            let window: id = msg_send![event, window];
            let icon_rect = get_tray_rect(window);

            // cursor position
            let mouse_location: NSPoint = msg_send![class!(NSEvent), mouseLocation];

            let event = TrayIconEvent {
                id: TrayIconId(id_str.to_string()),
                position: crate::dpi::PhysicalPosition::new(
                    mouse_location.x,
                    flip_window_screen_coordinates(mouse_location.y),
                ),
                icon_rect,
                click_type: click_event,
            };

            TrayIconEvent::send(event);

            let status_item = *this.get_ivar::<id>(TRAY_STATUS_ITEM);
            let button: id = msg_send![status_item, button];

            let menu_on_left_click = *this.get_ivar::<bool>(TRAY_MENU_ON_LEFT_CLICK);
            if click_event == ClickType::Right
                || (menu_on_left_click && click_event == ClickType::Left)
            {
                let menu = *this.get_ivar::<id>(TRAY_MENU);
                let has_items = if menu == nil {
                    false
                } else {
                    let num: NSInteger = msg_send![menu, numberOfItems];
                    num > 0
                };
                if has_items {
                    let _: () = msg_send![button, performClick: nil];
                } else {
                    let _: () = msg_send![button, highlight: YES];
                }
            } else {
                let _: () = msg_send![button, highlight: YES];
            }
        }

        TRAY_CLASS = decl.register();
    });

    unsafe { TRAY_CLASS }
}

fn get_tray_rect(window: id) -> Rect {
    let frame = unsafe { NSWindow::frame(window) };
    let scale_factor = unsafe { NSWindow::backingScaleFactor(window) };

    Rect {
        size: crate::dpi::LogicalSize::new(frame.size.width, frame.size.height)
            .to_physical(scale_factor),
        position: crate::dpi::LogicalPosition::new(
            frame.origin.x,
            flip_window_screen_coordinates(frame.origin.y),
        )
        .to_physical(scale_factor),
    }
}

/// Core graphics screen coordinates are relative to the top-left corner of
/// the so-called "main" display, with y increasing downwards - which is
/// exactly what we want in Winit.
///
/// However, `NSWindow` and `NSScreen` changes these coordinates to:
/// 1. Be relative to the bottom-left corner of the "main" screen.
/// 2. Be relative to the bottom-left corner of the window/screen itself.
/// 3. Have y increasing upwards.
///
/// This conversion happens to be symmetric, so we only need this one function
/// to convert between the two coordinate systems.
fn flip_window_screen_coordinates(y: f64) -> f64 {
    CGDisplay::main().pixels_high() as f64 - y
}
