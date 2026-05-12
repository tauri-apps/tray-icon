---
"tray-icon": patch
---

Fix the tray menu silently failing to pop up on macOS when the click happens on a secondary display while a full-screen Space is active there. The previous implementation synthesised a button click via `performClick(None)`, which carries no NSEvent context — AppKit's status-item popup logic relies on that context to resolve the active screen/Space and silently no-ops when it resolves to the wrong one. The popup is now driven directly through `NSStatusItem.popUpStatusItemMenu`, which positions the menu off the status item's own button frame and works in every monitor/Space combination.
