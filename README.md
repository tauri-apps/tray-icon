tray-icon lets you create tray icons for desktop applications.

# Example

Create a tray icon without a menu.

```rs
use tray_icon::TrayIconBuilder;

let tray_icon = TrayIconBuilder::new()
    .with_tooltip("system-tray - tray icon library!")
    .with_icon(icon)
    .build()
    .unwrap();
```

# Example

Create a tray icon with a menu.

```rs
use tray_icon::{TrayIconBuilder, menu::Menu};

let tray_menu = Menu::new();
let tray_icon = TrayIconBuilder::new()
    .with_menu(Box::new(tray_menu))
    .with_tooltip("system-tray - tray icon library!")
    .with_icon(icon)
    .build()
    .unwrap();
```

# Processing tray events

You can use `tray_event_receiver` to get a reference to the `TrayEventReceiver`
which you can use to listen to events when a click happens on the tray icon
```rs
use tray_icon::tray_event_receiver;

if let Ok(event) = tray_event_receiver().try_recv() {
    println!("{:?}", event);
}
```

You can also listen for the menu events using `menu_event_listener` to get events for the tray context menu.

```rs
use tray_icon::{tray_event_receiver, menu::menu_event_receiver};

if let Ok(event) = tray_event_receiver().try_recv() {
    println!("tray event: {:?}", event);
}

if let Ok(event) = menu_event_receiver().try_recv() {
    println!("menu event: {:?}", event);
}
```