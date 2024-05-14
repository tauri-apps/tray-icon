---
"tray-icon": "minor"
---

This release contains **breaking change** to the event structs in order to be able to add new `Enter`, `Move` and `Leave` events:

- Changed `TrayIconEvent` to be an enum instead of a struct.
- Added new events for when the mouse enters, moves or leaves the tray icon region.
- Removed `ClickType` enum and replaced it with `MouseButton` enum.
- Added `MouseButtonState` enum.
