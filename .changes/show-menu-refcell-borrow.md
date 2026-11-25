---
"tray-icon": patch
---

On Windows and macOS, fix `TrayIcon::show_menu` keeping the tray icon borrowed while the menu is open, which made setters such as `set_icon` and `set_tooltip` panic with "RefCell already borrowed" when called from the event loop during that time.
