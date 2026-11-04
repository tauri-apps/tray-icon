// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod icon;
use std::{
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
};

pub(crate) use icon::PlatformIcon;

use crate::{icon::IconType, TrayIconAttributes, TrayIconId};
use libappindicator::{AppIndicator, AppIndicatorStatus};

pub struct TrayIcon {
    id: TrayIconId,
    indicator: AppIndicator,
    temp_dir_path: Option<PathBuf>,
    icon: TempIcon,
    counter: u32,
    menu: Option<Box<dyn muda::ContextMenu>>,
}

impl TrayIcon {
    pub fn new(id: TrayIconId, mut attrs: TrayIconAttributes) -> crate::Result<Self> {
        let temp_icon = TempIcon::new(attrs.temp_dir_path.as_ref(), &id, 0)?;

        let icon = match (attrs.icon.take(), attrs.native_icon.take()) {
            (Some(icon), _) => Some(IconType::Custom(icon)),
            (None, icon) => icon.map(IconType::Native),
        };
        let icon_name = indicator_icon_name(icon, &temp_icon.file)?;

        let mut indicator = AppIndicator::with_path(
            &format!("tray-icon tray app {}", id.as_ref()),
            &icon_name,
            &temp_icon.directory.to_string_lossy(),
        );
        indicator.set_status(AppIndicatorStatus::Active);

        if let Some(menu) = &attrs.menu {
            indicator.set_menu(&mut menu.gtk_context_menu());
        }

        if let Some(title) = attrs.title {
            indicator.set_label(title.as_str(), "");
        }

        Ok(Self {
            id,
            indicator,
            icon: temp_icon,
            temp_dir_path: attrs.temp_dir_path,
            counter: 0,
            menu: attrs.menu,
        })
    }
    pub fn set_icon(&mut self, icon: Option<IconType>) -> crate::Result<()> {
        self.counter += 1;

        let temp_icon = TempIcon::new(self.temp_dir_path.as_ref(), &self.id, self.counter)?;
        let icon_name = indicator_icon_name(icon, &temp_icon.file)?;

        let theme_path = temp_icon.directory.to_string_lossy();
        self.indicator.set_icon_theme_path(&theme_path);
        self.indicator.set_icon_full(&icon_name, "tray icon");

        // Retire the previous directory only after publishing the replacement.
        self.icon = temp_icon;

        Ok(())
    }

    pub fn set_menu(&mut self, menu: Option<Box<dyn crate::menu::ContextMenu>>) {
        if let Some(menu) = &menu {
            self.indicator.set_menu(&mut menu.gtk_context_menu());
        }
        self.menu = menu;
    }

    pub fn set_tooltip<S: AsRef<str>>(&mut self, _tooltip: Option<S>) -> crate::Result<()> {
        Ok(())
    }

    pub fn set_title<S: AsRef<str>>(&mut self, title: Option<S>) {
        self.indicator
            .set_label(title.as_ref().map(|t| t.as_ref()).unwrap_or(""), "");
    }

    pub fn set_visible(&mut self, visible: bool) -> crate::Result<()> {
        if visible {
            self.indicator.set_status(AppIndicatorStatus::Active);
        } else {
            self.indicator.set_status(AppIndicatorStatus::Passive);
        }

        Ok(())
    }

    pub fn set_temp_dir_path<P: AsRef<Path>>(&mut self, path: Option<P>) {
        self.temp_dir_path = path.map(|p| p.as_ref().to_path_buf());
    }

    pub fn rect(&self) -> Option<crate::Rect> {
        None
    }

    pub fn app_indicator(&self) -> &AppIndicator {
        &self.indicator
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        self.indicator.set_status(AppIndicatorStatus::Passive);
    }
}

/// Returns the icon name to pass to AppIndicator, writing a custom icon to
/// `icon_path` as a PNG first.
fn indicator_icon_name(icon: Option<IconType>, icon_path: &Path) -> crate::Result<String> {
    match icon {
        Some(IconType::Native(icon)) => Ok(icon.freedesktop_name().to_owned()),
        Some(IconType::Custom(icon)) => {
            icon.inner.write_to_png(icon_path)?;
            let stem = icon_path.file_stem().unwrap();
            Ok(stem.to_string_lossy().into_owned())
        }
        None => Ok(String::new()),
    }
}

/// A private directory containing one PNG. Also removes partial files when
/// preparing an icon fails, without affecting the currently displayed icon.
struct TempIcon {
    directory: PathBuf,
    file: PathBuf,
}

impl TempIcon {
    /// Creates a fresh icon directory under one of the following dirs:
    /// 1. If `temp_icon_dir` is `Some` use that.
    /// 2. `$XDG_RUNTIME_DIR/tray-icon`
    /// 3. `/tmp/tray-icon`
    fn new(
        temp_icon_dir: Option<&PathBuf>,
        id: &TrayIconId,
        counter: u32,
    ) -> std::io::Result<Self> {
        let parent_path = match temp_icon_dir.as_ref() {
            Some(path) => path.to_path_buf(),
            None => dirs::runtime_dir()
                .unwrap_or_else(std::env::temp_dir)
                .join("tray-icon"),
        };

        std::fs::create_dir_all(&parent_path)?;
        let parent_path = parent_path.canonicalize()?;

        // Like Chromium's AppIndicator implementation, use a new search directory
        // for every image so rapid updates are not hidden by icon-theme caches.
        // https://chromium.googlesource.com/chromium/src/+/8df08cf1a988931e952fb85e811418bfe587d050/chrome/browser/ui/libgtk2ui/app_indicator_icon.cc
        loop {
            let directory_id = crate::COUNTER.next();
            let directory_name = format!("tray-icon-{}-{}", std::process::id(), directory_id);
            let directory = parent_path.join(directory_name);
            match std::fs::DirBuilder::new().mode(0o700).create(&directory) {
                Ok(()) => {
                    let file = directory.join(format!("tray-icon-{}-{}.png", id.as_ref(), counter));
                    return Ok(Self { directory, file });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
    }
}

impl Drop for TempIcon {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

#[test]
fn temp_icon_path_preference_order() {
    let runtime_dir = option_env!("XDG_RUNTIME_DIR");
    let override_dir = PathBuf::from("/tmp/tao-tests");

    let icon1 = TempIcon::new(Some(&override_dir), &"00".into(), 00).unwrap();
    let icon2 = TempIcon::new(None, &"00".into(), 00).unwrap();
    std::env::remove_var("XDG_RUNTIME_DIR");
    let icon3 = TempIcon::new(None, &"00".into(), 00).unwrap();

    assert_eq!(icon1.directory.parent().unwrap(), override_dir);
    if let Some(runtime_dir) = runtime_dir {
        std::env::set_var("XDG_RUNTIME_DIR", runtime_dir);
        assert_eq!(
            icon2.directory.parent().unwrap(),
            PathBuf::from(format!("{}/tray-icon", runtime_dir))
        );
    }

    assert_eq!(
        icon3.directory.parent().unwrap(),
        PathBuf::from("/tmp/tray-icon")
    );
}
