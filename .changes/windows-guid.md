---
"tray-icon": minor
---

Add `TrayIconBuilder::with_guid` / `TrayIconAttributes::guid` to register the icon with a stable `NOTIFYICONDATA.guidItem` on Windows, so the user's "always show in taskbar" setting survives the executable being replaced by an update.
