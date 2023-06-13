// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod icon;
mod util;
use std::ptr;

use once_cell::sync::Lazy;
use windows_sys::{
    s,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        UI::{
            Shell::{
                DefSubclassProc, SetWindowSubclass, Shell_NotifyIconGetRect, Shell_NotifyIconW,
                NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW,
                NOTIFYICONIDENTIFIER,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, GetCursorPos, RegisterClassW,
                RegisterWindowMessageA, SendMessageW, SetForegroundWindow, TrackPopupMenu,
                CW_USEDEFAULT, HICON, HMENU, TPM_BOTTOMALIGN, TPM_LEFTALIGN, WM_DESTROY,
                WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_RBUTTONUP, WNDCLASSW, WS_EX_LAYERED,
                WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_OVERLAPPED,
            },
        },
    },
};

use crate::{icon::Icon, menu, ClickEvent, Rectangle, TrayEvent, TrayIconAttributes};

pub(crate) use self::icon::WinIcon as PlatformIcon;

const TRAY_SUBCLASS_ID: usize = 6001;
const WM_USER_TRAYICON: u32 = 6002;
const WM_USER_UPDATE_TRAYMENU: u32 = 6003;
const WM_USER_UPDATE_TRAYICON: u32 = 6004;
const WM_USER_SHOW_TRAYICON: u32 = 6005;
const WM_USER_HIDE_TRAYICON: u32 = 6006;
const WM_USER_UPDATE_TRAYTOOLTIP: u32 = 6007;

/// When the taskbar is created, it registers a message with the "TaskbarCreated" string and then broadcasts this message to all top-level windows
/// When the application receives this message, it should assume that any taskbar icons it added have been removed and add them again.
static S_U_TASKBAR_RESTART: Lazy<u32> =
    Lazy::new(|| unsafe { RegisterWindowMessageA(s!("TaskbarCreated")) });

struct TrayLoopData {
    id: u32,
    hwnd: HWND,
    hpopupmenu: Option<HMENU>,
    icon: Option<Icon>,
    tooltip: Option<String>,
}

pub struct TrayIcon {
    hwnd: HWND,
    menu: Option<Box<dyn menu::ContextMenu>>,
    id: u32,
}

impl TrayIcon {
    pub fn new(id: u32, attrs: TrayIconAttributes) -> crate::Result<Self> {
        let class_name = util::encode_wide("tray_icon_app");
        unsafe {
            let hinstance = util::get_instance_handle();

            unsafe extern "system" fn call_default_window_proc(
                hwnd: HWND,
                msg: u32,
                wparam: WPARAM,
                lparam: LPARAM,
            ) -> LRESULT {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }

            let wnd_class = WNDCLASSW {
                lpfnWndProc: Some(call_default_window_proc),
                lpszClassName: class_name.as_ptr(),
                hInstance: hinstance,
                ..std::mem::zeroed()
            };

            RegisterClassW(&wnd_class);

            let hwnd = CreateWindowExW(
                WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_LAYERED |
            // WS_EX_TOOLWINDOW prevents this window from ever showing up in the taskbar, which
            // we want to avoid. If you remove this style, this window won't show up in the
            // taskbar *initially*, but it can show up at some later point. This can sometimes
            // happen on its own after several hours have passed, although this has proven
            // difficult to reproduce. Alternatively, it can be manually triggered by killing
            // `explorer.exe` and then starting the process back up.
            // It is unclear why the bug is triggered by waiting for several hours.
            WS_EX_TOOLWINDOW,
                class_name.as_ptr(),
                ptr::null(),
                WS_OVERLAPPED,
                CW_USEDEFAULT,
                0,
                CW_USEDEFAULT,
                0,
                HWND::default(),
                HMENU::default(),
                hinstance,
                std::ptr::null_mut(),
            );
            if hwnd == 0 {
                return Err(crate::Error::OsError(std::io::Error::last_os_error()));
            }

            let hicon = attrs.icon.as_ref().map(|i| i.inner.as_raw_handle());

            if !register_tray_icon(hwnd, id, &hicon, &attrs.tooltip) {
                return Err(crate::Error::OsError(std::io::Error::last_os_error()));
            }

            if let Some(menu) = &attrs.menu {
                menu.attach_menu_subclass_for_hwnd(hwnd);
            }

            // tray-icon event handler
            let traydata = TrayLoopData {
                id,
                hwnd,
                hpopupmenu: attrs.menu.as_ref().map(|m| m.hpopupmenu()),
                icon: attrs.icon,
                tooltip: attrs.tooltip,
            };
            SetWindowSubclass(
                hwnd,
                Some(tray_subclass_proc),
                TRAY_SUBCLASS_ID,
                Box::into_raw(Box::new(traydata)) as _,
            );

            Ok(Self {
                hwnd,
                id,
                menu: attrs.menu,
            })
        }
    }

    pub fn set_icon(&mut self, icon: Option<Icon>) -> crate::Result<()> {
        unsafe {
            let mut nid = NOTIFYICONDATAW {
                uFlags: NIF_ICON,
                hWnd: self.hwnd,
                uID: self.id,
                ..std::mem::zeroed()
            };

            if let Some(hicon) = icon.as_ref().map(|i| i.inner.as_raw_handle()) {
                nid.hIcon = hicon;
            }

            if Shell_NotifyIconW(NIM_MODIFY, &mut nid as _) == 0 {
                return Err(crate::Error::OsError(std::io::Error::last_os_error()));
            }

            // send the new icon to the subclass proc to store it in the tray data
            SendMessageW(
                self.hwnd,
                WM_USER_UPDATE_TRAYICON,
                Box::into_raw(Box::new(icon)) as _,
                0,
            );
        }

        Ok(())
    }

    pub fn set_menu(&mut self, menu: Option<Box<dyn menu::ContextMenu>>) {
        if let Some(menu) = &self.menu {
            menu.detach_menu_subclass_from_hwnd(self.hwnd);
        }

        unsafe {
            // send the new menu to the subclass proc where we will update there
            SendMessageW(
                self.hwnd,
                WM_USER_UPDATE_TRAYMENU,
                Box::into_raw(Box::new(menu.as_ref().map(|m| m.hpopupmenu()))) as _,
                0,
            );
        }

        self.menu = menu;
    }

    pub fn set_tooltip<S: AsRef<str>>(&mut self, tooltip: Option<S>) -> crate::Result<()> {
        unsafe {
            let mut nid = NOTIFYICONDATAW {
                uFlags: NIF_TIP,
                hWnd: self.hwnd,
                uID: self.id,
                ..std::mem::zeroed()
            };
            if let Some(tooltip) = &tooltip {
                let mut wide = util::encode_wide(tooltip.as_ref());
                wide.resize(128, 0);
                // nid.szTip.copy_from_slice(&wide); has misalginment issues on x86
                for i in 0..128 {
                    nid.szTip[i] = wide[i];
                }
            }

            if Shell_NotifyIconW(NIM_MODIFY, &mut nid as _) == 0 {
                return Err(crate::Error::OsError(std::io::Error::last_os_error()));
            }

            // send the new tooltip to the subclass proc to store it in the tray data
            SendMessageW(
                self.hwnd,
                WM_USER_UPDATE_TRAYTOOLTIP,
                Box::into_raw(Box::new(tooltip.map(|t| t.as_ref().to_string()))) as _,
                0,
            );
        }

        Ok(())
    }

    pub fn set_title<S: AsRef<str>>(&mut self, _title: Option<S>) {}

    pub fn set_visible(&mut self, visible: bool) -> crate::Result<()> {
        unsafe {
            SendMessageW(
                self.hwnd,
                if visible {
                    WM_USER_SHOW_TRAYICON
                } else {
                    WM_USER_HIDE_TRAYICON
                },
                0,
                0,
            );
        }

        Ok(())
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        unsafe {
            remove_tray_icon(self.hwnd, self.id);

            if let Some(menu) = &self.menu {
                menu.detach_menu_subclass_from_hwnd(self.hwnd);
            }

            // destroy the hidden window used by the tray
            DestroyWindow(self.hwnd);
        }
    }
}

unsafe extern "system" fn tray_subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    subclass_input_ptr: usize,
) -> LRESULT {
    let subclass_input_ptr = subclass_input_ptr as *mut TrayLoopData;
    let mut subclass_input = &mut *(subclass_input_ptr);

    match msg {
        WM_DESTROY => {
            drop(Box::from_raw(subclass_input_ptr));
        }
        WM_USER_UPDATE_TRAYMENU => {
            let hpopupmenu = Box::from_raw(wparam as *mut Option<isize>);
            subclass_input.hpopupmenu = *hpopupmenu;
        }
        WM_USER_UPDATE_TRAYICON => {
            let icon = Box::from_raw(wparam as *mut Option<Icon>);
            subclass_input.icon = *icon;
        }
        WM_USER_SHOW_TRAYICON => {
            register_tray_icon(
                subclass_input.hwnd,
                subclass_input.id,
                &subclass_input
                    .icon
                    .as_ref()
                    .map(|i| i.inner.as_raw_handle()),
                &subclass_input.tooltip,
            );
        }
        WM_USER_HIDE_TRAYICON => {
            remove_tray_icon(subclass_input.hwnd, subclass_input.id);
        }
        WM_USER_UPDATE_TRAYTOOLTIP => {
            let tooltip = Box::from_raw(wparam as *mut Option<String>);
            subclass_input.tooltip = *tooltip;
        }
        _ if msg == *S_U_TASKBAR_RESTART => {
            register_tray_icon(
                subclass_input.hwnd,
                subclass_input.id,
                &subclass_input
                    .icon
                    .as_ref()
                    .map(|i| i.inner.as_raw_handle()),
                &subclass_input.tooltip,
            );
        }
        WM_USER_TRAYICON
            if matches!(
                lparam as u32,
                WM_LBUTTONUP | WM_RBUTTONUP | WM_LBUTTONDBLCLK
            ) =>
        {
            let nid = NOTIFYICONIDENTIFIER {
                hWnd: hwnd,
                cbSize: std::mem::size_of::<NOTIFYICONIDENTIFIER>() as _,
                uID: subclass_input.id,
                ..std::mem::zeroed()
            };
            let mut icon_rect = RECT {
                left: 0,
                bottom: 0,
                right: 0,
                top: 0,
            };
            Shell_NotifyIconGetRect(&nid, &mut icon_rect);

            let mut cursor = POINT { x: 0, y: 0 };
            GetCursorPos(&mut cursor as _);

            let x = cursor.x as f64;
            let y = cursor.y as f64;

            let event = match lparam as u32 {
                WM_LBUTTONUP => ClickEvent::Left,
                WM_RBUTTONUP => ClickEvent::Right,
                WM_LBUTTONDBLCLK => ClickEvent::Double,
                _ => unreachable!(),
            };

            TrayEvent::send(crate::TrayEvent {
                id: subclass_input.id,
                x,
                y,
                icon_rect: Rectangle {
                    left: icon_rect.left as f64,
                    right: icon_rect.right as f64,
                    bottom: icon_rect.bottom as f64,
                    top: icon_rect.top as f64,
                },
                event,
            });

            if lparam as u32 == WM_RBUTTONUP {
                if let Some(menu) = subclass_input.hpopupmenu {
                    show_tray_menu(hwnd, menu, cursor.x, cursor.y);
                }
            }
        }
        _ => {}
    }

    DefSubclassProc(hwnd, msg, wparam, lparam)
}

unsafe fn show_tray_menu(hwnd: HWND, menu: HMENU, x: i32, y: i32) {
    // bring the hidden window to the foreground so the pop up menu
    // would automatically hide on click outside
    SetForegroundWindow(hwnd);
    TrackPopupMenu(
        menu,
        // align bottom / right, maybe we could expose this later..
        TPM_BOTTOMALIGN | TPM_LEFTALIGN,
        x,
        y,
        0,
        hwnd,
        std::ptr::null_mut(),
    );
}

unsafe fn register_tray_icon(
    hwnd: HWND,
    tray_id: u32,
    hicon: &Option<HICON>,
    tooltip: &Option<String>,
) -> bool {
    let mut h_icon = 0;
    let mut flags = NIF_MESSAGE;
    let mut sz_tip: [u16; 128] = [0; 128];

    if let Some(hicon) = hicon {
        flags |= NIF_ICON;
        h_icon = *hicon;
    }

    if let Some(tooltip) = tooltip {
        let mut tip = util::encode_wide(tooltip);
        tip.resize(128, 0);

        flags |= NIF_TIP;
        sz_tip.copy_from_slice(&tip)
    }

    let mut nid = NOTIFYICONDATAW {
        uFlags: flags,
        hWnd: hwnd,
        uID: tray_id,
        uCallbackMessage: WM_USER_TRAYICON,
        hIcon: h_icon,
        szTip: sz_tip,
        ..std::mem::zeroed()
    };

    Shell_NotifyIconW(NIM_ADD, &mut nid as _) == 1
}

unsafe fn remove_tray_icon(hwnd: HWND, id: u32) {
    let mut nid = NOTIFYICONDATAW {
        uFlags: NIF_ICON,
        hWnd: hwnd,
        uID: id,
        ..std::mem::zeroed()
    };

    if Shell_NotifyIconW(NIM_DELETE, &mut nid as _) == 0 {
        eprintln!("Error removing system tray icon");
    }
}
