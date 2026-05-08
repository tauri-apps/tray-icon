---
"tray-icon": minor
---

Add `TrayIconBuilder::with_autosave_name` (and corresponding `TrayIconAttributes::autosave_name` field) so callers can hand AppKit an `NSStatusItem.setAutosaveName` key on macOS. With the autosave name set, AppKit persists the user's ⌘+drag position across launches via `NSUserDefaults`. Without it, the status item resets to the default leftmost slot every launch — which on notched MacBooks (M1+ Pro/Max/14"/16") collides with the camera cutout. No-op on Linux / Windows.
