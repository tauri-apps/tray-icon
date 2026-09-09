// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::Path;

use crate::{icon::Icon, icon::RgbaIcon, TrayIconAttributes, TrayIconId};

#[derive(Debug, Clone)]
pub struct PlatformIcon;

impl PlatformIcon {
    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, crate::BadIcon> {
        let _ = RgbaIcon::from_rgba(rgba, width, height)?;
        Ok(Self)
    }
}

pub struct TrayIcon;

impl TrayIcon {
    pub fn new(_id: TrayIconId, _attrs: TrayIconAttributes) -> crate::Result<Self> {
        Ok(Self)
    }

    pub fn set_icon(&mut self, _icon: Option<Icon>) -> crate::Result<()> {
        Ok(())
    }

    pub fn set_menu(&mut self, _menu: Option<Box<dyn crate::menu::ContextMenu>>) {}

    pub fn set_tooltip<S: AsRef<str>>(&mut self, _tooltip: Option<S>) -> crate::Result<()> {
        Ok(())
    }

    pub fn set_title<S: AsRef<str>>(&mut self, _title: Option<S>) {}

    pub fn set_visible(&mut self, _visible: bool) -> crate::Result<()> {
        Ok(())
    }

    pub fn set_temp_dir_path<P: AsRef<Path>>(&mut self, _path: Option<P>) {}

    pub fn rect(&self) -> Option<crate::Rect> {
        None
    }
}
