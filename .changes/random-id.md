---
tray-icon: fix
---

The default `id` is now randomly generated instead of using a counter to prevent issues when multiple open apps use the `tray-icon` crate on Linux.
