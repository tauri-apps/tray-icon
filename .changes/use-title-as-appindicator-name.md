---
"tray-icon": patch
---

Use app title as `AppIndicator` name on Linux instead of the hardcoded `"tray-icon tray app"` string. This gives each app a unique indicator identity and prevents icon collisions when multiple apps use the crate.
