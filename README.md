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

## Example

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

## Processing tray events

You can use `TrayEvent::receiver` to get a reference to the `TrayEventReceiver`
which you can use to listen to events when a click happens on the tray icon
```rs
use tray_icon::TrayEvent;

if let Ok(event) = TrayEvent::receiver().try_recv() {
    println!("{:?}", event);
}
```

You can also listen for the menu events using `TrayEvent::receiver` to get events for the tray context menu.

```rs
use tray_icon::{TrayEvent, menu::{MenuEvent}};

if let Ok(event) = TrayEvent::receiver().try_recv() {
    println!("tray event: {:?}", event);
}

if let Ok(event) = MenuEvent::receiver().try_recv() {
    println!("menu event: {:?}", event);
}
```

## License

Apache-2.0/MIT
