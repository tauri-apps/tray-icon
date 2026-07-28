// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod icon;
use std::path::{Path, PathBuf};

use crate::icon::Icon;
pub(crate) use icon::PlatformIcon;

use crate::{TrayIconAttributes, TrayIconId};
use libappindicator::{AppIndicator, AppIndicatorStatus};

pub struct TrayIcon {
    id: TrayIconId,
    indicator: AppIndicator,
    temp_dir_path: Option<PathBuf>,
    counter: u32,
    icon_cache: Vec<CachedIcon>,
    menu: Option<Box<dyn muda::ContextMenu>>,
}

struct CachedIcon {
    icon: PlatformIcon,
    theme_path: PathBuf,
    icon_path: PathBuf,
}

impl CachedIcon {
    fn ensure_file(&self) -> crate::Result<()> {
        if self.icon_path.try_exists()? {
            return Ok(());
        }

        std::fs::create_dir_all(&self.theme_path)?;
        let temp_path = self.icon_path.with_extension("png.tmp");
        let result = (|| {
            self.icon.write_to_png(&temp_path)?;
            std::fs::rename(&temp_path, &self.icon_path)?;
            Ok(())
        })();

        if result.is_err() {
            let _ = std::fs::remove_file(temp_path);
        }

        result
    }
}

impl TrayIcon {
    pub fn new(id: TrayIconId, attrs: TrayIconAttributes) -> crate::Result<Self> {
        let mut indicator = AppIndicator::new(&format!("tray-icon tray app {}", id.as_ref()), "");
        indicator.set_status(AppIndicatorStatus::Active);

        let (parent_path, icon_path) = temp_icon_path(attrs.temp_dir_path.as_ref(), &id, 0)?;

        let mut icon_cache = Vec::new();
        if let Some(icon) = attrs.icon {
            icon.inner.write_to_png(&icon_path)?;
            icon_cache.push(CachedIcon {
                icon: icon.inner,
                theme_path: parent_path.clone(),
                icon_path: icon_path.clone(),
            });
        }

        indicator.set_icon_theme_path(&parent_path.to_string_lossy());
        indicator.set_icon_full(&icon_path.to_string_lossy(), "icon");

        if let Some(menu) = &attrs.menu {
            indicator.set_menu(&mut menu.gtk_context_menu());
        }

        if let Some(title) = attrs.title {
            indicator.set_label(title.as_str(), "");
        }

        Ok(Self {
            id,
            indicator,
            temp_dir_path: attrs.temp_dir_path,
            counter: 0,
            icon_cache,
            menu: attrs.menu,
        })
    }
    pub fn set_icon(&mut self, icon: Option<Icon>) -> crate::Result<()> {
        let (parent_path, icon_path) = match icon {
            Some(icon) => {
                if let Some(cached) = cached_icon(&self.icon_cache, &icon.inner) {
                    cached.ensure_file()?;
                    (cached.theme_path.clone(), cached.icon_path.clone())
                } else {
                    self.counter += 1;
                    let (parent_path, icon_path) =
                        temp_icon_path(self.temp_dir_path.as_ref(), &self.id, self.counter)?;
                    icon.inner.write_to_png(&icon_path)?;
                    self.icon_cache.push(CachedIcon {
                        icon: icon.inner,
                        theme_path: parent_path.clone(),
                        icon_path: icon_path.clone(),
                    });
                    (parent_path, icon_path)
                }
            }
            None => {
                self.counter += 1;
                temp_icon_path(self.temp_dir_path.as_ref(), &self.id, self.counter)?
            }
        };

        self.indicator
            .set_icon_theme_path(&parent_path.to_string_lossy());
        self.indicator
            .set_icon_full(&icon_path.to_string_lossy(), "tray icon");

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
        // AppIndicator consumers load icon paths asynchronously, so cached files
        // must remain available until the indicator is destroyed.
        for cached in &self.icon_cache {
            let _ = std::fs::remove_file(&cached.icon_path);
        }
    }
}

fn cached_icon<'a>(icon_cache: &'a [CachedIcon], icon: &PlatformIcon) -> Option<&'a CachedIcon> {
    icon_cache.iter().find(|cached| cached.icon == *icon)
}

/// Generates an icon path in one of the following dirs:
/// 1. If `temp_icon_dir` is `Some` use that.
/// 2. `$XDG_RUNTIME_DIR/tray-icon`
/// 3. `/tmp/tray-icon`
fn temp_icon_path(
    temp_icon_dir: Option<&PathBuf>,
    id: &TrayIconId,
    counter: u32,
) -> std::io::Result<(PathBuf, PathBuf)> {
    let parent_path = match temp_icon_dir.as_ref() {
        Some(path) => path.to_path_buf(),
        None => dirs::runtime_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("tray-icon"),
    };

    std::fs::create_dir_all(&parent_path)?;
    let icon_path = parent_path.join(format!("tray-icon-{}-{}.png", id.as_ref(), counter));
    Ok((parent_path, icon_path))
}

#[test]
fn temp_icon_path_preference_order() {
    let runtime_dir = option_env!("XDG_RUNTIME_DIR");
    let override_dir = PathBuf::from("/tmp/tao-tests");

    let (dir1, _file1) = temp_icon_path(Some(&override_dir), &"00".into(), 00).unwrap();
    let (dir2, _file1) = temp_icon_path(None, &"00".into(), 00).unwrap();
    std::env::remove_var("XDG_RUNTIME_DIR");
    let (dir3, _file2) = temp_icon_path(None, &"00".into(), 00).unwrap();

    assert_eq!(dir1, override_dir);
    if let Some(runtime_dir) = runtime_dir {
        std::env::set_var("XDG_RUNTIME_DIR", runtime_dir);
        assert_eq!(dir2, PathBuf::from(format!("{}/tray-icon", runtime_dir)));
    }

    assert_eq!(dir3, PathBuf::from("/tmp/tray-icon"));
}

#[test]
fn cached_icon_matches_icon_contents() {
    let icon = PlatformIcon::from_rgba(vec![0, 0, 0, 255], 1, 1).unwrap();
    let other_icon = PlatformIcon::from_rgba(vec![255, 255, 255, 255], 1, 1).unwrap();
    let cached = CachedIcon {
        icon: icon.clone(),
        theme_path: PathBuf::from("/tmp/tray-icon"),
        icon_path: PathBuf::from("/tmp/tray-icon/cached.png"),
    };
    let icon_cache = vec![cached];

    assert_eq!(
        cached_icon(&icon_cache, &icon).map(|cached| cached.icon_path.as_path()),
        Some(Path::new("/tmp/tray-icon/cached.png"))
    );
    assert!(cached_icon(&icon_cache, &other_icon).is_none());
}

#[test]
fn cached_icon_restores_a_missing_file() {
    let theme_path = std::env::temp_dir().join(format!("tray-icon-test-{}", std::process::id()));
    let icon_path = theme_path.join("cached.png");
    let cached = CachedIcon {
        icon: PlatformIcon::from_rgba(vec![0, 0, 0, 255], 1, 1).unwrap(),
        theme_path: theme_path.clone(),
        icon_path: icon_path.clone(),
    };

    let _ = std::fs::remove_dir_all(&theme_path);
    cached.ensure_file().unwrap();
    assert!(icon_path.is_file());

    std::fs::remove_file(&icon_path).unwrap();
    cached.ensure_file().unwrap();
    assert!(icon_path.is_file());

    std::fs::remove_dir_all(theme_path).unwrap();
}
