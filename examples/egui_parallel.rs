#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::process::exit;
use eframe::egui;
use tray_icon::{
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconEvent, TrayIconBuilder, TrayIcon
};


fn main() -> Result<(), eframe::Error> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/icon.png");
    let icon = load_icon(std::path::Path::new(path));

    let menu_items = vec![MenuItem::new("Quit", true, None)];

    // This is to provide the items ids to the event processing thread.
    let mut menu_items_id = Vec::new();
    for item in &menu_items {
        menu_items_id.push(item.id().clone());
    }

    // In the case of having more than one item to process, this could be a
    // simple way to having a distinguish between items.
    enum MenuItemsCmd {
        Exit = 0,
    }

    // Since egui uses winit under the hood and doesn't use gtk on Linux, and we
    // need gtk for the tray icon to show up, we need to spawn a thread where we
    // initialize gtk and create the tray_icon
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    std::thread::spawn(|| {
        gtk::init().unwrap();
        let _tray_icon = create_tray_icon(icon, menu_items);

        gtk::main();
    });

    #[cfg(not(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    let _tray_icon = create_tray_icon(icon, menu_items);

    // Thread that will process all the menu item events made by the user. This
    // thread will be destroyed when the main thread ends by design.
    std::thread::spawn(move || {
        let ids = menu_items_id;
        loop {
            if let Ok(event) = MenuEvent::receiver().recv() {
                if event.id == ids[MenuItemsCmd::Exit as usize] {
                    println!("Exit pressed!");
                    exit(0);
                }
            }
        }
    });

    let eframe_run_result = eframe::run_native(
        "My egui App",
        eframe::NativeOptions::default(),
        Box::new(move |_cc| {
            let app_struc = Box::<MyApp>::default();

            Ok(app_struc)
        }),
    );

    return eframe_run_result;
}


fn create_tray_icon(icon: tray_icon::Icon, menu_items: Vec<MenuItem>) -> TrayIcon {
    // Create the tray menu
    let tray_menu = Menu::new();

    // Append those items that doesn't need to interact with the rest of
    // the UI code
    tray_menu.append_items(&[
        &PredefinedMenuItem::about(
            None,
            Some(AboutMetadata {
                name: Some("egui example".to_string()),
                copyright: Some("Copyright egui example".to_string()),
                ..Default::default()
            }),
        ),
        &PredefinedMenuItem::separator(),
    ]).expect("Error creating the tray icon menu.");

    for item in menu_items {
        tray_menu.append(&item).unwrap();
    }

    // Create the tray icon and add the menu
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Some tray example text :-)")
        .with_icon(icon)
        .build()
        .unwrap();

    return tray_icon;
}


struct MyApp {
    name: String,
    age: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Those printing function of the tray event won't work when the window
        // isn't both openend and focused, as the egui event loop won't be running.

        if let Ok(event) = TrayIconEvent::receiver().try_recv() {
            println!("tray event: {event:?}");
        }

        if let Ok(event) = MenuEvent::receiver().try_recv() {
            println!("menu event: {event:?}");
        }

        // Design of the egui window
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
        });
    }
}

fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
