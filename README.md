tray-icon lets you create tray icons for desktop applications.

## Platforms supported:

- Windows
- macOS
- Linux (gtk Only)

## Platform-specific notes:

- On Windows and Linux, an event loop must be running on the thread, on Windows, a win32 event loop and on Linux, a gtk event loop. It doesn't need to be the main thread but you have to create the tray icon on the same thread as the event loop.
- On macOS, an event loop must be running on the main thread so you also need to create the tray icon on the main thread.

### Cargo Features

- `common-controls-v6`: Use `TaskDialogIndirect` API from `ComCtl32.dll` v6 on Windows for showing the predefined `About` menu item dialog.
- `libxdo`: Enables linking to `libxdo` which is used for the predfined `Copy`, `Cut`, `Paste` and `SelectAll` menu item, see https://github.com/tauri-apps/muda#cargo-features

## Dependencies (Linux Only)

On Linux, `gtk` and `libappindicator` or `libayatnat-appindicator` are used to create the tray icon, so make sure to install them on your system.

#### Arch Linux / Manjaro:

```sh
pacman -S gtk3 libappindicator-gtk3 #or libayatana-appindicator
```

#### Debian / Ubuntu:

```sh
sudo apt install libgtk-3-dev libappindicator3-dev #or libayatana-appindicator3-dev
```

if you use `tray_icon::muda` module, make sure to checkout https://github.com/tauri-apps/muda#dependencies

## Examples

#### Create a tray icon without a menu.

```rs
use tray_icon::TrayIconBuilder;

let tray_icon = TrayIconBuilder::new()
    .with_tooltip("system-tray - tray icon library!")
    .with_icon(icon)
    .build()
    .unwrap();
```

#### Create a tray icon with a menu.

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
