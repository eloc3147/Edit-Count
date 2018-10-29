extern crate iron;
extern crate staticfile;
extern crate mount;

mod ui_server;
mod directory_watcher;

use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<Error>> {
    // Scan directories
    let counts = Arc::new(Mutex::new(directory_watcher::Counts::new()));

    let group = directory_watcher::DirectoryGroup {
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
