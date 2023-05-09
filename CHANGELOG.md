# Changelog

## \[0.5.1]

- [`ff7f7bc`](https://www.github.com/tauri-apps/tray-icon/commit/ff7f7bc4400a6f7aa0b5c025c85ab6c4f89e9109)([#40](https://www.github.com/tauri-apps/tray-icon/pull/40)) Fix building for `i686-pc-windows-msvc` target.

## \[0.5.0]

- On macOS, fix `set_visible(false)` still occupying space on the system menu bar.
  - [71f9d29](https://www.github.com/tauri-apps/tray-icon/commit/71f9d292dd69b498e57fcebeb76ad6a1365144cd) fix(macos): remove tray icon when `set_visible(false)` ([#37](https://www.github.com/tauri-apps/tray-icon/pull/37)) on 2023-04-18

## \[0.4.4]

- Make Rectangle's point fields public.
  - [12a0daf](https://www.github.com/tauri-apps/tray-icon/commit/12a0daf92352fbecddd7b0afdfc0c633232fb15c) Make Rectangle's point fields public. ([#33](https://www.github.com/tauri-apps/tray-icon/pull/33)) on 2023-03-23

## \[0.4.3]

- Update documentation.
  - [258b49a](https://www.github.com/tauri-apps/tray-icon/commit/258b49aaebd81b6e4327cca1a1a0a2d9bb64188a) docs: update docs on 2023-02-08
  - [3293885](https://www.github.com/tauri-apps/tray-icon/commit/3293885ae5ef19e14f2fe1baaf4d35719f3b3344) Apply Version Updates From Current Changes ([#22](https://www.github.com/tauri-apps/tray-icon/pull/22)) on 2023-02-08
  - [e58a6ce](https://www.github.com/tauri-apps/tray-icon/commit/e58a6cecfffa63096d459429c5d31ec5b3475a9b) docs: document menu and icon relation on Linux on 2023-02-12

## \[0.4.2]

- Update docs.
  - [258b49a](https://www.github.com/tauri-apps/tray-icon/commit/258b49aaebd81b6e4327cca1a1a0a2d9bb64188a) docs: update docs on 2023-02-08

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
