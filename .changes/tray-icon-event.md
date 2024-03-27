---
"tray-icon": "minor"
---

Added `dpi` module and changed position and sizes in `TrayIconEvent` to use the new `dpi` module:

- Removed `TrayIconEvent.x` and `TrayIconEvent.y` and replaced with `TrayIconEvent.position`
- Replaced `Rectangle` type with `Rect` which has just two fields `position` and `size`.
