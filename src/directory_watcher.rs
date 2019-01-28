pub mod counter;
pub mod crawler;
pub mod directory_layout;
pub mod listener;

use self::counter::Counter;
use self::crawler::Crawler;
use self::listener::{Listener, ListenerEvent};
use crate::settings::Settings;
use crate::worker::Worker;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;

#[derive(Debug)]
pub struct DirectoryWatcher {
    settings: Arc<Settings>,
    cue_tx: Sender<CountUpdateEvent>,
}

impl DirectoryWatcher {
    pub fn new(settings: Arc<Settings>, cue_tx: Sender<CountUpdateEvent>) -> DirectoryWatcher {
        DirectoryWatcher { settings, cue_tx }
    }
}

impl Worker for DirectoryWatcher {
    type W = DirectoryWatcher;
    const NAME: &'static str = "Directory Watcher";

    fn work(self) {
        let (listener_tx, listener_rx) = channel();
        let (due_tx, due_rx) = channel();

        let listener = Listener::new(
            self.settings.watch_frequency,
            self.settings.directory_layout.clone(),
            listener_tx,
        )
        .start();

        let crawler = Crawler::new(self.settings.directory_layout.clone(), due_tx.clone()).start();

        let counter = Counter::new(due_rx, self.cue_tx);
        let counter = counter.start();

        while let Ok(listener_event) = listener_rx.recv() {
            match listener_event {
                ListenerEvent::Exist(path) => {
                    due_tx.send(DirectoryUpdateEvent::Exist(path)).unwrap()
                }
                ListenerEvent::Remove(path) => {
                    due_tx.send(DirectoryUpdateEvent::Remove(path)).unwrap()
                }
                ListenerEvent::Reload => {
                    unimplemented!();
                }
            }
        }

        counter.wait();
        listener.wait();
        crawler.wait();
    }
}

#[derive(Debug)]
pub enum DirectoryUpdateEvent {
    Exist(PathBuf),
    Remove(PathBuf),
    Set(SetEvent),
}

#[derive(Debug)]
pub struct SetEvent {
    group_name: String,
    album_name: String,
    tipe: GroupType,
    files: Vec<OsString>,
}

#[derive(Debug)]
pub struct CountUpdateEvent {
    group_name: String,
    album_name: String,
    count: Count,
}

#[derive(Debug)]
pub struct Count {
    total: usize,
    raw: usize,
    render: usize,
}

#[derive(Debug, Copy, Clone)]
pub enum GroupType {
    Raw,
    Render,
}
