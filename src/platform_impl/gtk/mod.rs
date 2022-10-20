mod icon;
use std::path::{Path, PathBuf};

use crate::icon::Icon;
pub(crate) use icon::PlatformIcon;

use crate::TrayIconAttributes;
use libappindicator::{AppIndicator, AppIndicatorStatus};

pub struct TrayIcon {
    id: u32,
    indicator: AppIndicator,
    temp_dir_path: Option<PathBuf>,
    path: PathBuf,
}

impl TrayIcon {
    pub fn new(id: u32, attrs: TrayIconAttributes) -> Result<Self, ()> {
        let mut indicator = AppIndicator::new("system_tray tray app", "");
        indicator.set_status(AppIndicatorStatus::Active);

        let (parent_path, icon_path) = temp_icon_path(attrs.temp_dir_path.as_ref(), id)
            .expect("Failed to create a temp folder for icon");

        if let Some(icon) = attrs.icon {
            icon.inner.write_to_png(&icon_path);
        }

        indicator.set_icon_theme_path(&parent_path.to_string_lossy());
        indicator.set_icon_full(&icon_path.to_string_lossy(), "icon");

        if let Some(menu) = attrs.menu {
            indicator.set_menu(&mut menu.gtk_context_menu());
        }

        Ok(Self {
            id,
            indicator,
            path: icon_path,
            temp_dir_path: attrs.temp_dir_path,
        })
    }
    pub fn set_icon(&mut self, icon: Option<Icon>) {
        let _ = std::fs::remove_file(&self.path);

        let (parent_path, icon_path) = temp_icon_path(self.temp_dir_path.as_ref(), self.id)
            .expect("Failed to create a temp folder for icon");

        if let Some(icon) = icon {
            icon.inner.write_to_png(&icon_path);
        }

        self.indicator
            .set_icon_theme_path(&parent_path.to_string_lossy());
        self.indicator
            .set_icon_full(&icon_path.to_string_lossy(), "icon");
        self.path = icon_path;
    }
    pub fn set_menu(&mut self, menu: Option<Box<dyn crate::menu::ContextMenu>>) {
        if let Some(menu) = menu {
            self.indicator.set_menu(&mut menu.gtk_context_menu());
        }
    }
    pub fn set_tooltip<S: AsRef<str>>(&mut self, _tooltip: Option<S>) {}
    pub fn set_temp_dir_path<P: AsRef<Path>>(&mut self, path: Option<P>) {
        self.temp_dir_path = path.map(|p| p.as_ref().to_path_buf());
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        self.indicator.set_status(AppIndicatorStatus::Passive);
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Generates an icon path in one of the following dirs:
/// 1. If `temp_icon_dir` is `Some` use that.
/// 2. `$XDG_RUNTIME_DIR/tao`
/// 3. `/tmp/tao`
fn temp_icon_path(temp_icon_dir: Option<&PathBuf>, id: u32) -> std::io::Result<(PathBuf, PathBuf)> {
    let parent_path = match temp_icon_dir.as_ref() {
        Some(path) => path.to_path_buf(),
        None => dirs_next::runtime_dir()
            .unwrap_or_else(|| std::env::temp_dir())
            .join("tao"),
    };

    std::fs::create_dir_all(&parent_path)?;
    let icon_path = parent_path.join(format!("tray-icon-{}.png", id));
    Ok((parent_path, icon_path))
}

#[test]
fn temp_icon_path_preference_order() {
    let runtime_dir = option_env!("XDG_RUNTIME_DIR");
    let override_dir = PathBuf::from("/tmp/tao-tests");

    let (dir1, _file1) = temp_icon_path(Some(&override_dir), 00).unwrap();
    let (dir2, _file1) = temp_icon_path(None, 00).unwrap();
    std::env::remove_var("XDG_RUNTIME_DIR");
    let (dir3, _file2) = temp_icon_path(None, 00).unwrap();

    assert_eq!(dir1, override_dir);
    if let Some(runtime_dir) = runtime_dir {
        std::env::set_var("XDG_RUNTIME_DIR", runtime_dir);
        assert_eq!(dir2, PathBuf::from(format!("{}/tao", runtime_dir)));
    }

    assert_eq!(dir3, PathBuf::from("/tmp/tao"));
}
