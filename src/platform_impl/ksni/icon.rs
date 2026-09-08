// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::icon::{BadIcon, RgbaIcon};

#[derive(Debug, Clone)]
pub struct PlatformIcon(ksni::Icon);

impl PlatformIcon {
    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, BadIcon> {
        let icon = RgbaIcon::from_rgba(rgba, width, height)?;
        let mut data = icon.rgba;
        for pixel in data.as_chunks_mut::<4>().0 {
            pixel.rotate_right(1);
        }

        Ok(Self(ksni::Icon {
            width: icon.width as i32,
            height: icon.height as i32,
            data,
        }))
    }
}

impl From<PlatformIcon> for ksni::Icon {
    fn from(icon: PlatformIcon) -> Self {
        icon.0
    }
}

#[cfg(test)]
mod tests {
    use super::PlatformIcon;

    #[test]
    fn converts_rgba_to_network_order_argb() {
        let icon = PlatformIcon::from_rgba(vec![1, 2, 3, 4], 1, 1).unwrap();
        let icon: ksni::Icon = icon.into();

        assert_eq!(icon.width, 1);
        assert_eq!(icon.height, 1);
        assert_eq!(icon.data, [4, 1, 2, 3]);
    }
}
