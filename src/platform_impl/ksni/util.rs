// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// taken from https://github.com/bilelmoussaoui/ashpd/blob/a849791fb9e48efbd32a9c980eb769e1250162a2/client/src/helpers.rs
// and https://github.com/bilelmoussaoui/ashpd/blob/a849791fb9e48efbd32a9c980eb769e1250162a2/client/src/lib.rs#L72-L79

use std::{io::Read, sync::OnceLock};

static IS_SANDBOXED: OnceLock<bool> = OnceLock::new();

fn is_flatpak() -> bool {
    std::path::PathBuf::from("/.flatpak-info").exists()
}

fn is_snap() -> bool {
    let pid = std::process::id();
    let path = format!("/proc/{pid}/cgroup");
    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(_) => return false,
    };

    let mut buffer = String::new();
    match file.read_to_string(&mut buffer) {
        Ok(_) => cgroup_v2_is_snap(&buffer),
        Err(_) => false,
    }
}

fn cgroup_v2_is_snap(cgroups: &str) -> bool {
    cgroups
        .lines()
        .map(|line| {
            let (n, rest) = line.split_once(':')?;
            // Check that n is a number.
            n.parse::<u32>().ok()?;
            let unit = match rest.split_once(':') {
                Some(("", unit)) => Some(unit),
                Some(("freezer", unit)) => Some(unit),
                Some(("name=systemd", unit)) => Some(unit),
                _ => None,
            }?;
            let scope = std::path::Path::new(unit).file_name()?.to_str()?;

            Some(scope.starts_with("snap."))
        })
        .any(|x| x.unwrap_or(false))
}

/// Check whether the application is running inside a sandbox.
///
/// The function checks whether the file `/.flatpak-info` exists, or if the app
/// is running as a snap. As the return value of this function will not change during the
/// runtime of a program; it is cached for future calls.
pub(crate) fn is_sandboxed() -> bool {
    if let Some(cached_value) = IS_SANDBOXED.get() {
        return *cached_value;
    }
    let new_value = is_flatpak() || is_snap();

    *IS_SANDBOXED.get_or_init(|| new_value)
}
