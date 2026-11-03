// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// taken from https://github.com/rust-windowing/winit/blob/92fdf5ba85f920262a61cee4590f4a11ad5738d1/src/platform_impl/windows/icon.rs

use std::{fmt, io, mem, path::Path, sync::Arc};

use windows_sys::{
    core::PCWSTR,
    Win32::UI::WindowsAndMessaging::{
        CreateIcon, DestroyIcon, LoadImageW, HICON, IMAGE_ICON, LR_DEFAULTSIZE, LR_LOADFROMFILE,
    },
};

use crate::icon::*;

use super::util;

impl Pixel {
    fn convert_to_bgra(&mut self) {
        mem::swap(&mut self.r, &mut self.b);
    }
}

impl RgbaIcon {
    fn into_windows_icon(self) -> Result<WinIcon, BadIcon> {
        let (rgba, width, height) = rgba_for_tray(self.rgba, self.width, self.height);
        let pixel_count = rgba.len() / PIXEL_SIZE;
        let mut and_mask = Vec::with_capacity(pixel_count);
        let pixels =
            unsafe { std::slice::from_raw_parts_mut(rgba.as_ptr() as *mut Pixel, pixel_count) };
        for pixel in pixels {
            and_mask.push(pixel.a.wrapping_sub(u8::MAX)); // invert alpha channel
            pixel.convert_to_bgra();
        }
        assert_eq!(and_mask.len(), pixel_count);
        let handle = unsafe {
            CreateIcon(
                std::ptr::null_mut(),
                width as i32,
                height as i32,
                1,
                (PIXEL_SIZE * 8) as u8,
                and_mask.as_ptr(),
                rgba.as_ptr(),
            )
        };
        if !handle.is_null() {
            Ok(WinIcon::from_handle(handle))
        } else {
            Err(BadIcon::OsError(io::Error::last_os_error()))
        }
    }
}

#[derive(Debug)]
struct RaiiIcon {
    handle: HICON,
}

#[derive(Clone)]
pub(crate) struct WinIcon {
    inner: Arc<RaiiIcon>,
}

unsafe impl Send for WinIcon {}

impl WinIcon {
    pub fn as_raw_handle(&self) -> HICON {
        self.inner.handle
    }

    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, BadIcon> {
        let rgba_icon = RgbaIcon::from_rgba(rgba, width, height)?;
        rgba_icon.into_windows_icon()
    }

    pub(crate) fn from_handle(handle: HICON) -> Self {
        Self {
            #[allow(clippy::arc_with_non_send_sync)]
            inner: Arc::new(RaiiIcon { handle }),
        }
    }

    pub(crate) fn from_path<P: AsRef<Path>>(
        path: P,
        size: Option<(u32, u32)>,
    ) -> Result<Self, BadIcon> {
        // `None` is the notification-area small icon. A zero size plus
        // `LR_DEFAULTSIZE` would load `SM_CXICON` instead, and the shell would
        // scale that down.
        let (width, height) = size.unwrap_or_else(util::get_tray_icon_size);

        let wide_path = util::encode_wide(path.as_ref());

        let handle = unsafe {
            LoadImageW(
                std::ptr::null_mut(),
                wide_path.as_ptr(),
                IMAGE_ICON,
                width as i32,
                height as i32,
                LR_DEFAULTSIZE | LR_LOADFROMFILE,
            )
        };
        if !handle.is_null() {
            Ok(WinIcon::from_handle(handle as HICON))
        } else {
            Err(BadIcon::OsError(io::Error::last_os_error()))
        }
    }

    fn from_resource_inner_name(name: PCWSTR, size: Option<(u32, u32)>) -> Result<Self, BadIcon> {
        let (width, height) = size.unwrap_or_else(util::get_tray_icon_size);
        let handle = unsafe {
            LoadImageW(
                util::get_instance_handle(),
                name,
                IMAGE_ICON,
                width as i32,
                height as i32,
                LR_DEFAULTSIZE,
            )
        };
        if !handle.is_null() {
            Ok(WinIcon::from_handle(handle as HICON))
        } else {
            Err(BadIcon::OsError(io::Error::last_os_error()))
        }
    }

    pub(crate) fn from_resource(
        resource_id: u16,
        size: Option<(u32, u32)>,
    ) -> Result<Self, BadIcon> {
        Self::from_resource_inner_name(resource_id as PCWSTR, size)
    }

    pub(crate) fn from_resource_name(
        resource_name: &str,
        size: Option<(u32, u32)>,
    ) -> Result<Self, BadIcon> {
        let wide_name = util::encode_wide(resource_name);
        Self::from_resource_inner_name(wide_name.as_ptr(), size)
    }
}

impl Drop for RaiiIcon {
    fn drop(&mut self) {
        unsafe { DestroyIcon(self.handle) };
    }
}

impl fmt::Debug for WinIcon {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        (*self.inner).fmt(formatter)
    }
}

/// Fit `rgba` into the notification-area slot without letting the shell scale it.
fn rgba_for_tray(rgba: Vec<u8>, width: u32, height: u32) -> (Vec<u8>, u32, u32) {
    let (target_w, target_h) = util::get_tray_icon_size();
    fit_rgba_into(rgba, width, height, target_w, target_h)
}

/// Scale `rgba` into `target_w` × `target_h`, preserving aspect ratio.
///
/// Equal sizes return the original buffer. A non-square source is letterboxed
/// on a transparent canvas so the shell does not stretch it.
fn fit_rgba_into(
    rgba: Vec<u8>,
    width: u32,
    height: u32,
    target_w: u32,
    target_h: u32,
) -> (Vec<u8>, u32, u32) {
    if width == 0 || height == 0 || target_w == 0 || target_h == 0 {
        return (rgba, width, height);
    }
    if width == target_w && height == target_h {
        return (rgba, width, height);
    }

    let scale = (target_w as f64 / width as f64).min(target_h as f64 / height as f64);
    let content_w = ((width as f64 * scale).round() as u32).clamp(1, target_w);
    let content_h = ((height as f64 * scale).round() as u32).clamp(1, target_h);
    let resized = if content_w == width && content_h == height {
        rgba
    } else {
        resample_rgba_area(&rgba, width, height, content_w, content_h)
    };
    if content_w == target_w && content_h == target_h {
        return (resized, target_w, target_h);
    }

    let mut canvas = vec![0u8; target_w as usize * target_h as usize * PIXEL_SIZE];
    let offset_x = (target_w - content_w) / 2;
    let offset_y = (target_h - content_h) / 2;
    let row_bytes = content_w as usize * PIXEL_SIZE;
    for y in 0..content_h {
        let src = y as usize * row_bytes;
        let dst = ((offset_y + y) * target_w + offset_x) as usize * PIXEL_SIZE;
        canvas[dst..dst + row_bytes].copy_from_slice(&resized[src..src + row_bytes]);
    }
    (canvas, target_w, target_h)
}

/// Area-average resample of straight RGBA8.
///
/// Coverage is accumulated in premultiplied alpha so a hard edge becomes a
/// partial-alpha edge instead of a stair step, and transparent pixels do not
/// darken the colors they are blended with.
fn resample_rgba_area(src: &[u8], src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> Vec<u8> {
    let mut dst = vec![0u8; dst_w as usize * dst_h as usize * PIXEL_SIZE];
    let src_w_f = src_w as f64;
    let src_h_f = src_h as f64;
    let dst_w_f = dst_w as f64;
    let dst_h_f = dst_h as f64;

    for y in 0..dst_h {
        let src_y0 = y as f64 * src_h_f / dst_h_f;
        let src_y1 = (y + 1) as f64 * src_h_f / dst_h_f;
        let y_start = src_y0.floor() as u32;
        let y_end = src_y1.ceil().min(src_h_f) as u32;

        for x in 0..dst_w {
            let src_x0 = x as f64 * src_w_f / dst_w_f;
            let src_x1 = (x + 1) as f64 * src_w_f / dst_w_f;
            let x_start = src_x0.floor() as u32;
            let x_end = src_x1.ceil().min(src_w_f) as u32;

            let mut acc = [0.0f64; 4];
            let mut covered = 0.0f64;
            for src_y in y_start..y_end {
                let y_cov = ((src_y + 1) as f64).min(src_y1) - (src_y as f64).max(src_y0);
                if y_cov <= 0.0 {
                    continue;
                }
                for src_x in x_start..x_end {
                    let x_cov = ((src_x + 1) as f64).min(src_x1) - (src_x as f64).max(src_x0);
                    if x_cov <= 0.0 {
                        continue;
                    }
                    let weight = x_cov * y_cov;
                    let index = (src_y as usize * src_w as usize + src_x as usize) * PIXEL_SIZE;
                    let alpha = src[index + 3] as f64 / 255.0;
                    acc[0] += src[index] as f64 * alpha * weight;
                    acc[1] += src[index + 1] as f64 * alpha * weight;
                    acc[2] += src[index + 2] as f64 * alpha * weight;
                    acc[3] += alpha * weight;
                    covered += weight;
                }
            }

            if covered > 0.0 && acc[3] > 0.0 {
                let index = (y as usize * dst_w as usize + x as usize) * PIXEL_SIZE;
                dst[index] = (acc[0] / acc[3]).round().clamp(0.0, 255.0) as u8;
                dst[index + 1] = (acc[1] / acc[3]).round().clamp(0.0, 255.0) as u8;
                dst[index + 2] = (acc[2] / acc[3]).round().clamp(0.0, 255.0) as u8;
                dst[index + 3] = (acc[3] / covered * 255.0).round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    dst
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_size_is_copied_unchanged() {
        let rgba = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let (out, width, height) = fit_rgba_into(rgba.clone(), 2, 1, 2, 1);
        assert_eq!((width, height), (2, 1));
        assert_eq!(out, rgba);
    }

    #[test]
    fn area_downsample_keeps_premultiplied_color() {
        // Opaque red beside a transparent pixel. Averaging the raw channels
        // would turn the red into dark red; coverage in premultiplied alpha
        // must keep the color and only lower the alpha.
        let rgba = vec![255, 0, 0, 255, 0, 0, 0, 0];
        let out = resample_rgba_area(&rgba, 2, 1, 1, 1);
        assert_eq!(out, vec![255, 0, 0, 128]);
    }

    #[test]
    fn area_downsample_softens_a_binary_edge() {
        let mut rgba = Vec::new();
        for alpha in [255u8, 255, 255, 0] {
            rgba.extend_from_slice(&[255, 255, 255, alpha]);
        }
        let out = resample_rgba_area(&rgba, 4, 1, 2, 1);
        assert_eq!(&out[0..4], &[255, 255, 255, 255]);
        assert_eq!(&out[4..8], &[255, 255, 255, 128]);
    }

    #[test]
    fn non_square_bitmap_is_letterboxed() {
        let rgba = vec![255, 0, 0, 255, 0, 255, 0, 255];
        let (out, width, height) = fit_rgba_into(rgba, 2, 1, 2, 2);
        assert_eq!((width, height), (2, 2));
        assert_eq!(&out[0..8], &[255, 0, 0, 255, 0, 255, 0, 255]);
        assert_eq!(&out[8..16], &[0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn zero_target_keeps_the_source() {
        let rgba = vec![1, 2, 3, 4];
        let (out, width, height) = fit_rgba_into(rgba.clone(), 1, 1, 0, 0);
        assert_eq!((width, height), (1, 1));
        assert_eq!(out, rgba);
    }

    #[test]
    fn tray_slot_is_reported() {
        let (width, height) = util::get_tray_icon_size();
        assert!(width > 0);
        assert!(height > 0);
    }
}
