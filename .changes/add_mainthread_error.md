---
"tray-icon": minor
---

Added a new variant `NotMainThread` to the `Error` enum, which is emitted on macOS when trying to create tray icons from a thread that is not the main thread.
