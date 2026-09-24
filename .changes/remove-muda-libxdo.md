---
"tray-icon": minor
---

**Breaking change** Remove the `muda-libxdo` Cargo feature, which was enabled by default. `muda` no longer has a `libxdo` feature: its predefined `Copy`, `Cut`, `Paste` and `SelectAll` items now work without one, and no longer need `libxdo-dev` to build. In a tray menu they act on the application's own active window, where they used to act on whichever window had the keyboard focus.
