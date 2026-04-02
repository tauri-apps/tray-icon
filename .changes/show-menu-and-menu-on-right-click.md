---
"tray-icon": minor
---

Added `with_menu_on_right_click` builder method, `set_show_menu_on_right_click` to control whether the context menu is shown on right click (analogous to the existing left click option), and `show_menu()` to programmatically display the tray menu. 

Together these enable dynamic menu workflows where the menu content is updated before being shown, for example by disabling automatic right-click menu, listening for the click event, updating items, and then calling `show_menu()`.
