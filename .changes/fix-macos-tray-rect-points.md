---
"tray-icon": patch
---

On macOS, the screen-coordinate flip used `CGDisplayPixelsHigh` where the incoming values are Cocoa points, skewing the reported tray icon rect and cursor position by the scale factor whenever the main display is Retina. Flip against the main display's bounds height in points instead.
