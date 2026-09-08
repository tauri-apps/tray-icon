// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{io::Cursor, sync::Arc};

use muda::{MenuItemKindSnapshot, SnapshotIcon};

use super::StatusNotifierTray;

pub(super) fn into_ksni_menu(
    items: &[MenuItemKindSnapshot],
) -> Vec<ksni::MenuItem<StatusNotifierTray>> {
    items.iter().filter_map(into_ksni_item).collect()
}

fn into_ksni_item(item: &MenuItemKindSnapshot) -> Option<ksni::MenuItem<StatusNotifierTray>> {
    match item {
        MenuItemKindSnapshot::MenuItem(item) => {
            let activate = Arc::clone(&item.activate);
            Some(
                ksni::menu::StandardItem {
                    label: to_ksni_mnemonic(&item.text()),
                    enabled: item.is_enabled(),
                    activate: Box::new(move |_| activate()),
                    ..Default::default()
                }
                .into(),
            )
        }
        MenuItemKindSnapshot::Check(item) => {
            let activate = Arc::clone(&item.activate);
            Some(
                ksni::menu::CheckmarkItem {
                    label: to_ksni_mnemonic(&item.text()),
                    enabled: item.is_enabled(),
                    checked: item.is_checked(),
                    activate: Box::new(move |_| activate()),
                    ..Default::default()
                }
                .into(),
            )
        }
        MenuItemKindSnapshot::Submenu(item) => {
            let (icon_name, icon_data) = icon_fields(item.icon().as_ref());
            Some(
                ksni::menu::SubMenu {
                    label: to_ksni_mnemonic(&item.text()),
                    enabled: item.is_enabled(),
                    icon_name,
                    icon_data,
                    submenu: into_ksni_menu(&item.items()),
                    ..Default::default()
                }
                .into(),
            )
        }
        MenuItemKindSnapshot::Predefined(item) if item.is_separator() => {
            Some(ksni::MenuItem::Separator)
        }
        MenuItemKindSnapshot::Icon(item) => {
            let (icon_name, icon_data) = icon_fields(item.icon().as_ref());
            let activate = Arc::clone(&item.activate);
            Some(
                ksni::menu::StandardItem {
                    label: to_ksni_mnemonic(&item.text()),
                    enabled: item.is_enabled(),
                    icon_name,
                    icon_data,
                    activate: Box::new(move |_| activate()),
                    ..Default::default()
                }
                .into(),
            )
        }
        // TODO: support some predefined items like "Quit" or "About"
        MenuItemKindSnapshot::Predefined(item) => Some(
            ksni::menu::StandardItem {
                label: to_ksni_mnemonic(&item.text()),
                enabled: false,
                ..Default::default()
            }
            .into(),
        ),
    }
}

fn to_ksni_mnemonic(text: &str) -> String {
    let mut converted = String::with_capacity(text.len());
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '_' => converted.push_str("__"),
            '&' if characters.peek() == Some(&'&') => {
                characters.next();
                converted.push('&');
            }
            '&' => converted.push('_'),
            _ => converted.push(character),
        }
    }
    converted
}

fn icon_fields(icon: Option<&SnapshotIcon>) -> (String, Vec<u8>) {
    match icon {
        Some(SnapshotIcon::Rgba {
            rgba,
            width,
            height,
        }) => (
            String::new(),
            encode_png(rgba, *width, *height).unwrap_or_default(),
        ),
        Some(SnapshotIcon::Native(icon)) => (icon.freedesktop_name().to_string(), Vec::new()),
        _ => Default::default(),
    }
}

fn encode_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, png::EncodingError> {
    let mut data = Vec::new();
    {
        let mut encoder = png::Encoder::new(Cursor::new(&mut data), width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(rgba)?;
    }
    Ok(data)
}
