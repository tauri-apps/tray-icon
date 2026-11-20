// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(target_os = "linux")]

use std::{
    error::Error,
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use tray_icon::{Icon, TrayIconBuilder, TrayIconStatus};
use zbus::{blocking::connection::Builder, message::Header};

struct Item {
    destination: String,
    path: String,
}

struct Watcher(Sender<Item>);

#[zbus::interface(name = "org.kde.StatusNotifierWatcher")]
impl Watcher {
    fn register_status_notifier_item(
        &self,
        service: &str,
        #[zbus(header)] header: Header<'_>,
    ) -> zbus::fdo::Result<()> {
        let item = if service.starts_with('/') {
            Item {
                destination: header
                    .sender()
                    .ok_or_else(|| zbus::fdo::Error::Failed("missing sender".into()))?
                    .to_string(),
                path: service.to_owned(),
            }
        } else {
            Item {
                destination: service.to_owned(),
                path: "/StatusNotifierItem".into(),
            }
        };
        self.0
            .send(item)
            .map_err(|error| zbus::fdo::Error::Failed(error.to_string()))
    }

    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn protocol_version(&self) -> i32 {
        0
    }

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> Vec<String> {
        Vec::new()
    }
}

enum Command {
    Status(TrayIconStatus),
    Visible(bool),
    Finished(Result<(), String>),
}

struct IconDirectory(PathBuf);

impl Drop for IconDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn inspect(
    registrations: Receiver<Item>,
    commands: &Sender<Command>,
) -> Result<(), Box<dyn Error>> {
    let item = registrations.recv_timeout(Duration::from_secs(10))?;
    let connection = Builder::session()?
        .method_timeout(Duration::from_secs(5))
        .build()?;
    let proxy = zbus::blocking::Proxy::new(
        &connection,
        item.destination.as_str(),
        item.path.as_str(),
        "org.kde.StatusNotifierItem",
    )?;
    let properties = zbus::blocking::Proxy::new(
        &connection,
        item.destination.as_str(),
        item.path.as_str(),
        "org.freedesktop.DBus.Properties",
    )?;
    // Read directly: SNI sends NewStatus, rather than requiring PropertiesChanged.
    let read_status = || -> Result<String, Box<dyn Error>> {
        let value: zbus::zvariant::OwnedValue =
            properties.call("Get", &("org.kde.StatusNotifierItem", "Status"))?;
        Ok(String::try_from(value)?)
    };
    if read_status()? != "Active" {
        return Err("new tray item did not start Active".into());
    }
    // Installing the match before requesting any transitions avoids a signal race.
    let mut signals = proxy.receive_signal("NewStatus")?;
    for (command, expected) in [
        (
            Command::Status(TrayIconStatus::NeedsAttention),
            "NeedsAttention",
        ),
        (Command::Status(TrayIconStatus::Active), "Active"),
        (Command::Status(TrayIconStatus::Passive), "Passive"),
        (
            Command::Status(TrayIconStatus::NeedsAttention),
            "NeedsAttention",
        ),
        (Command::Visible(true), "Active"),
        (
            Command::Status(TrayIconStatus::NeedsAttention),
            "NeedsAttention",
        ),
        (Command::Visible(false), "Passive"),
        (Command::Visible(true), "Active"),
    ] {
        commands.send(command)?;
        let signal = signals.next().ok_or("NewStatus stream closed")?;
        let observed = signal.body().deserialize::<String>()?;
        if observed != expected {
            return Err(format!("NewStatus: expected {expected}, received {observed}").into());
        }
        let observed = read_status()?;
        if observed != expected {
            return Err(format!("Status: expected {expected}, received {observed}").into());
        }
    }
    Ok(())
}

/// Run on a private session bus and test display, never the desktop's watcher:
/// TRAY_ICON_TEST_SESSION=1 dbus-run-session -- xvfb-run -a cargo test --test linux_status -- --ignored
#[test]
#[ignore = "requires a private D-Bus session and Xvfb; see the test command"]
fn status_property_and_signals_follow_status_and_visibility() -> Result<(), Box<dyn Error>> {
    assert_eq!(std::env::var("TRAY_ICON_TEST_SESSION").as_deref(), Ok("1"));
    let (registered, registrations) = mpsc::channel();
    let _watcher = Builder::session()?
        .name("org.kde.StatusNotifierWatcher")?
        .serve_at("/StatusNotifierWatcher", Watcher(registered))?
        .build()?;

    #[cfg(not(feature = "ksni"))]
    gtk::init()?;
    let unique = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let directory = IconDirectory(
        std::env::temp_dir().join(format!("tray-icon-status-{}-{unique}", std::process::id())),
    );
    std::fs::create_dir(&directory.0)?;
    let builder = TrayIconBuilder::new()
        .with_id("status-integration-test")
        .with_icon(Icon::from_rgba(vec![255, 255, 255, 255], 1, 1)?)
        .with_temp_dir_path(&directory.0);
    #[cfg(not(feature = "ksni"))]
    let builder = builder.with_menu(Box::new(tray_icon::menu::Menu::new()));
    let tray = builder.build()?;
    // A cloned public handle must update the same exported item.
    let handle = tray.clone();
    let (commands, receiver) = mpsc::channel();
    let inspector = thread::spawn(move || {
        let result = inspect(registrations, &commands).map_err(|error| error.to_string());
        let _ = commands.send(Command::Finished(result));
    });
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for SNI status/signals"
        );
        #[cfg(not(feature = "ksni"))]
        while gtk::events_pending() {
            gtk::main_iteration_do(false);
        }
        match receiver.recv_timeout(Duration::from_millis(5)) {
            Ok(Command::Status(status)) => handle.set_status(status)?,
            Ok(Command::Visible(visible)) => handle.set_visible(visible)?,
            Ok(Command::Finished(result)) => {
                inspector.join().expect("status inspector panicked");
                return result.map_err(Into::into);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(error) => return Err(error.into()),
        }
    }
}
