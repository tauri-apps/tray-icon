---
"tray-icon": patch
---

Rewrite the internals of the crate to use `objc2` instead of `objc`.

This should have no user-facing changes, other than improved memory safety, and less leaking.
