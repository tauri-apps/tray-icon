---
tray-icon: patch
---

Add platform specific methods to access the underlying native handles of the tray (similar to `TrayIcon::window_handle`):
- `TrayIcon::ns_status_item` for macOS
- `TrayIcon::app_indicator` for Linux.
