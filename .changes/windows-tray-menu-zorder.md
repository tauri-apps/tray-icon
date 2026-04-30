---
"tray-icon": patch
---

Fixed a Windows tray menu issue reported in #303 where the context menu could appear behind the taskbar after launching from the Start Menu by aligning the tray menu handling with the expected shell behavior, including showing the right-click menu on button release and posting `WM_NULL` after `TrackPopupMenu`.
