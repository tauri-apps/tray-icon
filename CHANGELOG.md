# Changelog

## \[0.4.1]

- Bump `muda` to `0.4` and `libappindicator` to `0.8`
  - [d92dd6d](https://www.github.com/tauri-apps/tray-icon/commit/d92dd6dc25d268befe9c14cfe193e1de10bc5717) chore(deps): update deps ([#17](https://www.github.com/tauri-apps/tray-icon/pull/17)) on 2023-01-26

## \[0.4.0]

- On macOS and Linux, add `TrayIconBuilder::with_title` and `TrayIcon::set_title` to optionally add a text next to the icon.
  - [6df6fc7](https://www.github.com/tauri-apps/tray-icon/commit/6df6fc78885204be5189b41527a39324851c9671) feat: add `with_title` and `set_title` ([#11](https://www.github.com/tauri-apps/tray-icon/pull/11)) on 2023-01-10
  - [b83f14e](https://www.github.com/tauri-apps/tray-icon/commit/b83f14ee66f9d3801535697c30f54bccc433cce1) chore: adjust change bumps on 2023-01-12
- Add `TrayIcon::set_visible`.
  - [ba4580e](https://www.github.com/tauri-apps/tray-icon/commit/ba4580ec8bd061a76575859b5ead8ec16e3b7817) feat: add `set_visible` ([#14](https://www.github.com/tauri-apps/tray-icon/pull/14)) on 2023-01-12
  - [b83f14e](https://www.github.com/tauri-apps/tray-icon/commit/b83f14ee66f9d3801535697c30f54bccc433cce1) chore: adjust change bumps on 2023-01-12

## \[0.3.0]

- Add `TrayEvent::set_event_handler` to set a handler for new tray events.
  - [9247abb](https://www.github.com/tauri-apps/tray-icon/commit/9247abb69ce297096b2c388d67b250509fe44efa) refactor: allow changing the menu event sender ([#8](https://www.github.com/tauri-apps/tray-icon/pull/8)) on 2023-01-03
- Update `muda` to `0.3`.
  - [9247abb](https://www.github.com/tauri-apps/tray-icon/commit/9247abb69ce297096b2c388d67b250509fe44efa) refactor: allow changing the menu event sender ([#8](https://www.github.com/tauri-apps/tray-icon/pull/8)) on 2023-01-03
  - [b64b57e](https://www.github.com/tauri-apps/tray-icon/commit/b64b57ec565dada4bc06201f5b4529725bb0009f) chore: update changefile on 2023-01-03
- **Breaking change** Remove `tray_event_receiver` function, use `TrayEvent::receiver` instead.
  - [9247abb](https://www.github.com/tauri-apps/tray-icon/commit/9247abb69ce297096b2c388d67b250509fe44efa) refactor: allow changing the menu event sender ([#8](https://www.github.com/tauri-apps/tray-icon/pull/8)) on 2023-01-03

## \[0.2.0]

- Update `muda` dependency to `0.2`.
  - [aa3aa1e](https://www.github.com/tauri-apps/tray-icon/commit/aa3aa1ec0bdcb48ecf9d17204809802c4e6559fc) chore: add change file on 2022-12-30

## \[0.1.1]

- Initial Release.
  - [0651773](https://www.github.com/tauri-apps/tray-icon/commit/0651773ad248d34141fbefc1c65a8889a90a8c9b) chore: prepare for initial release on 2022-12-05
