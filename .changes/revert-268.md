---
tray-icon: patch
---

This hotfix reverts https://github.com/tauri-apps/tray-icon/pull/268 because it caused `assertion failed: flush_paint_messages` panics.
