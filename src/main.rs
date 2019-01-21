mod directory_watcher;
mod settings;
mod ui_server;
mod worker;

use crate::directory_watcher::DirectoryWatcher;
use crate::settings::Settings;
use crate::ui_server::UIServer;
use crate::worker::Worker;
use app_dirs::{app_root, AppDataType, AppInfo};
use std::error::Error;
use std::sync::mpsc::channel;
use std::sync::Arc;

fn main() -> Result<(), Box<Error>> {
    const APP_INFO: AppInfo = AppInfo {
        name: "edit_count",
        author: "edit_count",
    };
    let settings_root = app_root(AppDataType::UserConfig, &APP_INFO)?;

    // Load settings
    let settings = Settings::new(&settings_root)?;
    let settings = Arc::new(settings);

    // Scan directories
    let (cue_tx, cue_rx) = channel();
    let watcher = DirectoryWatcher::new(settings.clone(), cue_tx).start();

    // Start web interface
    let address = String::from("127.0.0.1:2183");
    let _ = UIServer::launch(address.clone());
    println!("Server started at {}", address);

    while let Ok(event) = cue_rx.recv() {
        println!("{:?}", event);
    }

    watcher.wait();

    Ok(())
}
