---
"tray-icon": patch
---

Handle Windows tray icon creation when the taskbar is not ready by keeping the message window alive and re-registering on TaskbarCreated.
