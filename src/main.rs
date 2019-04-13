mod counter;
mod crawler;
mod directory_layout;
mod listener;
mod settings;
mod ui_server;
mod worker;

use crate::settings::Settings;
use crate::ui_server::UIServer;
use crate::worker::Worker;
use app_dirs::{app_root, AppDataType, AppInfo};
use failure::Error;
use std::sync::mpsc::channel;
use std::sync::Arc;

use crate::counter::Counter;
use crate::crawler::Crawler;
use crate::listener::{Listener, ListenerEvent};
use json::{object, JsonValue};
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug)]
pub enum DirectoryUpdateEvent {
    Exist(PathBuf),
    Remove(PathBuf),
    Set(SetEvent),
    Refresh,
}

#[derive(Debug)]
pub struct SetEvent {
    group_name: String,
    album_name: String,
    tipe: GroupType,
    files: Vec<OsString>,
}

#[derive(Debug, Clone)]
pub struct CountUpdateEvent {
    group_name: String,
    album_name: String,
    count: Count,
}

unsafe impl Send for CountUpdateEvent {}
unsafe impl Sync for CountUpdateEvent {}

impl Into<JsonValue> for CountUpdateEvent {
    fn into(self) -> JsonValue {
        object! {
            "group_name" => self.group_name,
            "album_name" => self.album_name,
            "total" => self.count.total,
            "raw" => self.count.raw,
            "render" => self.count.render
        }
    }
}

#[derive(Debug, Clone)]
pub struct Count {
    total: usize,
    raw: usize,
    render: usize,
}

unsafe impl Send for Count {}
unsafe impl Sync for Count {}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GroupType {
    Raw,
    Render,
}

fn main() -> Result<(), Error> {
    const APP_INFO: AppInfo = AppInfo {
        name: "edit_count",
        author: "edit_count",
    };

    // Load settings
    let settings_root = app_root(AppDataType::UserConfig, &APP_INFO)?;
    let settings = Settings::from(settings_root.join("settings.toml"))?;
    let settings = Arc::new(settings);

    // Create channels
    let (listener_tx, listener_rx) = channel();
    let (cue_tx, cue_rx) = channel();
    let (due_tx, due_rx) = channel();

    // Start workers
    let listener_handle = Listener::new(
        settings.watch_frequency,
        settings.directory_layout.clone(),
        listener_tx,
    )
    .start()?;

    let crawler_handle = Crawler::new(settings.directory_layout.clone(), due_tx.clone()).start()?;

    let counter = Counter::new(cue_tx, due_rx);

    let ui_server_handle = UIServer::new(
        settings.web_port,
        settings.ws_port,
        cue_rx,
        counter.get_handle(),
    )
    .start()?;

    let counter_handle = counter.start()?;

    println!("Server started at http://127.0.0.1:{}", settings.web_port);

    // Dispatch Listener events
    while let Ok(event) = listener_rx.recv() {
        match event {
            ListenerEvent::Exist(path) => {
                due_tx.send(DirectoryUpdateEvent::Exist(path))?;
            }
            ListenerEvent::Remove(path) => {
                due_tx.send(DirectoryUpdateEvent::Remove(path))?;
            }
            ListenerEvent::Rescan => {
                // TODO: Crawl directory structure
                unimplemented!();
            }
        }
    }

    // Wait for workers
    ui_server_handle.join()?;
    listener_handle.join()?;
    crawler_handle.join()?;
    counter_handle.join()?;

    Ok(())
}
