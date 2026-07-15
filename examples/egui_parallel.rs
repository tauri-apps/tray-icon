#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(not(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
)))]
use std::{cell::RefCell, rc::Rc};

use std::process::exit;
use std::sync::{Arc, Mutex};
use eframe::egui;
use tray_icon::{
    menu::{AboutMetadata, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem},
    TrayIconEvent, TrayIconBuilder, TrayIcon
};


fn main() -> Result<(), eframe::Error> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/icon.png");
    let icon = load_icon(std::path::Path::new(path));

    macro_rules! generate_menu_items {
        () => (vec![MenuItem::new("Quit", true, None)])
    }

    // This is to provide the items ids to the event processing thread.
    let menu_items_id = Arc::new(Mutex::new(Vec::<MenuId>::new()));

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
    ))]{
        let ids_gtk_thread = Arc::clone(&menu_items_id);

        std::thread::spawn(move || {
            gtk::init().unwrap();

            let menu_items = generate_menu_items!();
            let _tray_icon = create_tray_icon(icon, &menu_items);

            {
                let mut ids = ids_gtk_thread.lock().unwrap();
                for item in &menu_items {
                    ids.push(item.id().clone());
                }
            }

            gtk::main();
        });
    }


    #[cfg(not(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    let mut _tray_icon = Rc::new(RefCell::new(None));

    #[cfg(not(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    let tray_c = _tray_icon.clone();

    #[cfg(not(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    let menu_items;

    #[cfg(not(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))] {
        menu_items = generate_menu_items!();

        let ids_win_thread = Arc::clone(&menu_items_id);

        {
            let mut ids = ids_win_thread.lock().unwrap();
            for item in &menu_items {
                ids.push(item.id().clone());
            }
        }
    }

    // Thread that will process all the menu item events made by the user. This
    // thread will be destroyed when the main thread ends by design.
    //
    // The function 'recv()' is a locking one, and that means the loop will get
    // stuck until some event is generated, then it will continue running.
    let ids_loop_thread = Arc::clone(&menu_items_id);
    std::thread::spawn(move || {
        loop {
            if let Ok(event) = MenuEvent::receiver().recv() {
                let ids = ids_loop_thread.lock().unwrap();

                assert!((ids.len() > 0), "No IDs found to process in parrallel!");

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

            #[cfg(not(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )))]
            {
                tray_c
                    .borrow_mut()
                    .replace(create_tray_icon(icon, &menu_items));
            }

            Ok(app_struc)
        }),
    );

    return eframe_run_result;
}


fn create_tray_icon(icon: tray_icon::Icon, menu_items: &Vec<MenuItem>) -> TrayIcon {
    // Create the tray menu
    let tray_menu = Menu::new();

    // Append those static items that doesn't need to interact with the rest of
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

    // Append items that will be interacting with egui in some way or another.
    for item in menu_items {
        tray_menu.append(item).unwrap();
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

        // Print tray icon events.
        if let Ok(event) = TrayIconEvent::receiver().try_recv() {
            println!("tray event: {event:?}");
        }

        // Print tray icon menu events.
        // Those events already received by the parrallel thread won't be here,
        // and knowing this frame generating thread is locked to 60 FPS (or the
        // screen frequency), this is a lot slower, and rarely will get any of
        // those events.
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
