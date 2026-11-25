---
"tray-icon": minor
---

Add `Icon::from_rgba_with_scale`, which creates an icon drawn at a number of pixels per point. On macOS, a 32x32 icon with a scale factor of `2.0` is now drawn at 16x16 points and stays sharp on Retina displays. Windows and Linux ignore the scale factor.
