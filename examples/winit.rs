#![allow(unused)]

use system_tray::{
    menu::{menu_event_receiver, AboutMetadata, Menu, MenuItem, PredefinedMenuItem},
    tray_event_receiver, TrayIconBuilder,
};
use winit::event_loop::{ControlFlow, EventLoopBuilder};

fn main() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/icon.png");
    let icon = load_icon(std::path::Path::new(path));

    let event_loop = EventLoopBuilder::new().build();

    let tray_menu = Menu::new();

    let quit_i = MenuItem::new("Quit", true, None);
    tray_menu.append_items(&[
        &PredefinedMenuItem::about(
            None,
            Some(AboutMetadata {
                name: Some("winit".to_string()),
                copyright: Some("Copyright winit".to_string()),
                ..Default::default()
            }),
        ),
        &PredefinedMenuItem::separator(),
        &quit_i,
    ]);

    let mut tray_icon = Some(
        TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("winit - awesome windowing lib")
            .with_icon(icon)
            .build()
            .unwrap(),
    );

    let menu_channel = menu_event_receiver();
    let tray_channel = tray_event_receiver();

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == quit_i.id() {
                tray_icon.take();
                *control_flow = ControlFlow::Exit;
            }
            println!("{:?}", event);
        }

        if let Ok(event) = tray_channel.try_recv() {
            println!("{:?}", event);
        }
    })
}

fn load_icon(path: &std::path::Path) -> system_tray::icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    system_tray::icon::Icon::from_rgba(icon_rgba, icon_width, icon_height)
        .expect("Failed to open icon")
}
