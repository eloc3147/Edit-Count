extern crate app_dirs;
extern crate config;
extern crate iron;
extern crate mount;
extern crate regex;
extern crate serde;
extern crate staticfile;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

mod directory_watcher;
mod settings;
mod ui_server;

use app_dirs::{app_root, AppDataType, AppInfo};
use directory_watcher::{Counts, DirectoryGroup, DirectoryWatcher};
use settings::Settings;
use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<Error>> {
    const APP_INFO: AppInfo = AppInfo {
        name: "Edit Count",
        author: "Edit Count",
    };
    let settings_root = app_root(AppDataType::UserConfig, &APP_INFO)?;

    // Load settings
    let settings = Settings::new(&settings_root)?;

    /*
    // Start web interface
    let address = String::from("127.0.0.1:2183");
    let _ = ui_server::new(&address)?;
    println!("Server started at {}", address);
*/


    // Scan directories
    let watcher = DirectoryWatcher::new(settings.directory_layout);
    println!("{:#?}", watcher);
    let counts = Arc::new(Mutex::new(Counts::new()));

    let group = DirectoryGroup {
        name: String::from("Punk Wedding"),
        raw_dirs: vec![PathBuf::from("D:\\Photography\\RAWS\\2018\\Punk Wedding")],
        render_dirs: vec![PathBuf::from("D:\\Photos\\2018\\Punk Wedding")],
    };
    directory_watcher::update_counts(counts.clone(), group)?;

    for (key, value) in counts.lock().unwrap().iter() {
        println!("{}: {:#?}", key, value);
    }

    Ok(())
}
