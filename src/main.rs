mod directory_watcher;
mod settings;
mod ui_server;

use crate::directory_watcher::directory_layout::DirectoryLayout;
use crate::directory_watcher::DirectoryWatcher;
use crate::settings::Settings;
use crate::ui_server::UIServer;
use app_dirs::{app_root, AppDataType, AppInfo};
use std::error::Error;
use std::sync::Arc;

fn main() -> Result<(), Box<Error>> {
    const APP_INFO: AppInfo = AppInfo {
        name: "edit_count",
        author: "edit_count",
    };
    let settings_root = app_root(AppDataType::UserConfig, &APP_INFO)?;

    // Load settings
    let settings = Settings::new(&settings_root)?;

    // Start web interface
    let address = String::from("127.0.0.1:2183");
    let _ = UIServer::launch(address.clone());
    println!("Server started at {}", address);

    // Scan directories
    let layout: Arc<DirectoryLayout> = Arc::new(settings.directory_layout);
    let watcher = DirectoryWatcher::launch(settings.watch_frequency, layout.clone())?;
    watcher.join();

    Ok(())
}
