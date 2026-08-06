---
"tray-icon": patch
---

On macOS, dropping the last `TrayIcon` handle off the main thread now defers `NSStatusItem` teardown to the main dispatch queue instead of crashing with a BoardServices main-queue assertion (`EXC_BREAKPOINT`) on macOS 26.
