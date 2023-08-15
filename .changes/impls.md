---
"tray-icon": "patch"
---

Add `PartialEq<&str> for &TrayIconId` and `PartialEq<String> for &TrayIconId` implementations. Also add a blanket `From<T> for TrayIconId` where `T: ToString` implementation.
