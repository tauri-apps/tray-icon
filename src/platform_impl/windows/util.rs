// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::ops::{Deref, DerefMut};

use once_cell::sync::Lazy;
use windows_sys::{
    core::HRESULT,
    Win32::{
        Foundation::{FARPROC, HWND, S_OK},
        Graphics::Gdi::{
            GetDC, GetDeviceCaps, MonitorFromWindow, HMONITOR, LOGPIXELSX, MONITOR_DEFAULTTONEAREST,
        },
        System::LibraryLoader::{GetProcAddress, LoadLibraryW},
        UI::{
            HiDpi::{MDT_EFFECTIVE_DPI, MONITOR_DPI_TYPE},
            WindowsAndMessaging::{IsProcessDPIAware, ACCEL, WINDOW_LONG_PTR_INDEX},
        },
    },
};

pub fn encode_wide<S: AsRef<std::ffi::OsStr>>(string: S) -> Vec<u16> {
    std::os::windows::prelude::OsStrExt::encode_wide(string.as_ref())
        .chain(std::iter::once(0))
        .collect()
}

/// ACCEL wrapper to implement Debug
#[derive(Clone)]
#[repr(transparent)]
pub struct Accel(pub ACCEL);

impl std::fmt::Debug for Accel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ACCEL")
            .field("key", &self.0.key)
            .field("cmd", &self.0.cmd)
            .field("fVirt", &self.0.fVirt)
            .finish()
    }
}

impl Deref for Accel {
    type Target = ACCEL;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Accel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// taken from winit's code base
// https://github.com/rust-windowing/winit/blob/ee88e38f13fbc86a7aafae1d17ad3cd4a1e761df/src/platform_impl/windows/util.rs#L138
pub fn get_instance_handle() -> windows_sys::Win32::Foundation::HMODULE {
    // Gets the instance handle by taking the address of the
    // pseudo-variable created by the microsoft linker:
    // https://devblogs.microsoft.com/oldnewthing/20041025-00/?p=37483

    // This is preferred over GetModuleHandle(NULL) because it also works in DLLs:
    // https://stackoverflow.com/questions/21718027/getmodulehandlenull-vs-hinstance

    extern "C" {
        static __ImageBase: windows_sys::Win32::System::SystemServices::IMAGE_DOS_HEADER;
    }

    unsafe { &__ImageBase as *const _ as _ }
}

fn get_function_impl(library: &str, function: &str) -> FARPROC {
    let library = encode_wide(library);
    assert_eq!(function.chars().last(), Some('\0'));

    // Library names we will use are ASCII so we can use the A version to avoid string conversion.
    let module = unsafe { LoadLibraryW(library.as_ptr()) };
    if module == 0 {
        return None;
    }

    unsafe { GetProcAddress(module, function.as_ptr()) }
}

macro_rules! get_function {
    ($lib:expr, $func:ident) => {
        crate::platform_impl::platform::util::get_function_impl(
            $lib,
            concat!(stringify!($func), '\0'),
        )
        .map(|f| unsafe { std::mem::transmute::<_, $func>(f) })
    };
}

pub type GetDpiForMonitor = unsafe extern "system" fn(
    hmonitor: HMONITOR,
    dpi_type: MONITOR_DPI_TYPE,
    dpi_x: *mut u32,
    dpi_y: *mut u32,
) -> HRESULT;

static GET_DPI_FOR_MONITOR: Lazy<Option<GetDpiForMonitor>> =
    Lazy::new(|| get_function!("shcore.dll", GetDpiForMonitor));

pub const BASE_DPI: u32 = 96;
pub fn dpi_to_scale_factor(dpi: u32) -> f64 {
    dpi as f64 / BASE_DPI as f64
}

#[allow(non_snake_case)]
pub unsafe fn hwnd_dpi(hwnd: HWND) -> u32 {
    if let Some(GetDpiForMonitor) = *GET_DPI_FOR_MONITOR {
        // We are on Windows 8.1 or later.
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        if monitor == 0 {
            return BASE_DPI;
        }

        let mut dpi_x = 0;
        let mut dpi_y = 0;
        #[allow(clippy::unnecessary_cast)]
        if GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) == S_OK {
            dpi_x as u32
        } else {
            BASE_DPI
        }
    } else {
        let hdc = GetDC(hwnd);
        if hdc == 0 {
            return BASE_DPI;
        }

        // We are on Vista or later.
        if IsProcessDPIAware() == 1 {
            // If the process is DPI aware, then scaling must be handled by the application using
            // this DPI value.
            GetDeviceCaps(hdc, LOGPIXELSX as _) as u32
        } else {
            // If the process is DPI unaware, then scaling is performed by the OS; we thus return
            // 96 (scale factor 1.0) to prevent the window from being re-scaled by both the
            // application and the WM.
            BASE_DPI
        }
    }
}

#[inline(always)]
pub unsafe fn get_window_long(hwnd: HWND, nindex: WINDOW_LONG_PTR_INDEX) -> isize {
    #[cfg(target_pointer_width = "64")]
    return unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, nindex) };
    #[cfg(target_pointer_width = "32")]
    return unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongW(hwnd, nindex) as isize
    };
}

#[inline(always)]
pub unsafe fn set_window_long(
    hwnd: HWND,
    nindex: WINDOW_LONG_PTR_INDEX,
    dwnewlong: isize,
) -> isize {
    #[cfg(target_pointer_width = "64")]
    return unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(hwnd, nindex, dwnewlong)
    };
    #[cfg(target_pointer_width = "32")]
    return unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::SetWindowLongW(hwnd, nindex, dwnewlong as i32)
            as isize
    };
}
