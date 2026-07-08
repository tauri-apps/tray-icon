// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::icon::BadIcon;

#[derive(Debug, Clone)]
pub struct PlatformIcon {
    argb: Vec<u8>,
    width: i32,
    height: i32,
}

impl PlatformIcon {
    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, BadIcon> {
        let mut bytes = rgba;
        for pixel in bytes.chunks_exact_mut(4) {
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            let a = pixel[3];
            pixel[0] = a;
            pixel[1] = r;
            pixel[2] = g;
            pixel[3] = b;
        }

        Ok(Self {
            argb: bytes,
            width: width as i32,
            height: height as i32,
        })
    }

    pub fn into_rgba(self) -> (Vec<u8>, u32, u32) {
        let mut bytes = self.argb;
        for pixel in bytes.chunks_exact_mut(4) {
            let a = pixel[0];
            let r = pixel[1];
            let g = pixel[2];
            let b = pixel[3];
            pixel[0] = r;
            pixel[1] = g;
            pixel[2] = b;
            pixel[3] = a;
        }

        (bytes, self.width as u32, self.height as u32)
    }
}

impl From<PlatformIcon> for ksni::Icon {
    fn from(icon: PlatformIcon) -> Self {
        ksni::Icon {
            width: icon.width,
            height: icon.height,
            data: icon.argb,
        }
    }
}
