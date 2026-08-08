use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{TrayIconBuilder, TrayIconEvent};

fn main() {
    let menu = Menu::new();
    let open = MenuItem::new("Open", true, None);
    let quit = MenuItem::new("Quit", true, None);
    menu.append_items(&[&open, &quit]).unwrap();

    let icon =
        tray_icon::Icon::from_rgba(vec![0u8, 120, 255, 255].repeat(16 * 16), 16, 16).unwrap();

    let _tray = TrayIconBuilder::new()
        .with_id("ksni-e2e")
        .with_menu(Box::new(menu))
        .with_tooltip("ksni e2e")
        .with_icon(icon)
        .build()
        .unwrap();

    println!("tray up (id=ksni-e2e); listening 30s");

    let tray_rx = TrayIconEvent::receiver();
    let menu_rx = MenuEvent::receiver();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);

    while std::time::Instant::now() < deadline {
        if let Ok(e) = tray_rx.try_recv() {
            println!("TRAY EVENT: {e:?}");
        }
        if let Ok(e) = menu_rx.try_recv() {
            println!("MENU EVENT: {e:?}");
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
