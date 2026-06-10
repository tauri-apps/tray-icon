---
"tray-icon": patch
---

fix(windows): preserve tray icon visibility on Explorer restart

Track visibility state in TrayUserData and only re-register the tray icon
on `TaskbarCreated` if the icon was visible. Previously, a hidden tray
icon would incorrectly reappear when Windows Explorer restarted.

Also fixes error on close when `TrayIcon` is not visible.
