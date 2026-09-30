---
"tray-icon": minor
---

Add `TrayIcon::set_icon_templated` and `TrayIconBuilder::with_icon_templated` on macOS, which set the icon and draw it as a [template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc) image in one call, so the template flag can never be left describing an icon that has since been replaced.

Deprecate `TrayIcon::set_icon_as_template`, `TrayIcon::set_icon_with_as_template` and `TrayIconBuilder::with_icon_as_template` in favour of them. `TrayIcon::set_icon` now also clears `TrayIconAttributes::icon_is_template`, which used to keep saying `true` for an icon it no longer described.
