// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
mod platform;
#[cfg(any(
    all(target_os = "linux", feature = "gtk", not(feature = "linux-ksni")),
    all(target_os = "dragonfly", feature = "gtk"),
    all(target_os = "freebsd", feature = "gtk"),
    all(target_os = "netbsd", feature = "gtk"),
    all(target_os = "openbsd", feature = "gtk")
))]
#[path = "gtk/mod.rs"]
mod platform;
#[cfg(all(target_os = "linux", feature = "linux-ksni"))]
#[path = "linux/mod.rs"]
mod platform;
#[cfg(any(
    all(target_os = "linux", not(feature = "gtk"), not(feature = "linux-ksni")),
    all(target_os = "dragonfly", not(feature = "gtk")),
    all(target_os = "freebsd", not(feature = "gtk")),
    all(target_os = "netbsd", not(feature = "gtk")),
    all(target_os = "openbsd", not(feature = "gtk"))
))]
compile_error!(
    "Enable the `gtk` feature, or on Linux use `--no-default-features --features linux-ksni`."
);
#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
mod platform;

pub(crate) use self::platform::*;
