---
"tray-icon": patch
---

On Linux and BSD, the KSNI backend no longer fails to build a tray icon when no `org.kde.StatusNotifierWatcher` is on the session bus yet. The icon registers when the watcher appears, as the AppIndicator backend does. Before, an app that starts before the panel got an error and had no icon for the session.
