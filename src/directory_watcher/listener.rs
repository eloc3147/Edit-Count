use super::directory_layout::{DirectoryLayout, PathComponent};
use crate::worker::Worker;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;

pub struct Listener {
    watch_frequency: u64,
    layout: DirectoryLayout,
    listener_tx: Sender<ListenerEvent>,
}

impl Listener {
    pub fn new(
        watch_frequency: u64,
        layout: DirectoryLayout,
        listener_tx: Sender<ListenerEvent>,
    ) -> Listener {
        Listener {
            watch_frequency,
            layout,
            listener_tx,
        }
    }
}

impl Worker for Listener {
    type W = Listener;
    const NAME: &'static str = "Listener";
    fn work(mut self) {
        // Build list of all watched base paths
        let mut watch_dirs =
            Vec::with_capacity(self.layout.raw_dirs.len() + self.layout.render_dirs.len());

        for path in self
            .layout
            .raw_dirs
            .iter()
            .chain(self.layout.render_dirs.iter())
        {
            let mut path_cache = PathBuf::new();
            for component in path {
                match component {
                    PathComponent::Dir(dir) => path_cache.push(dir),
                    _ => break,
                }
            }
            watch_dirs.push(path_cache);
        }

        // Launch the filesystem listener
        let (watcher_tx, watcher_rx) = channel();
        let mut watcher = watcher(watcher_tx, Duration::from_millis(self.watch_frequency))
            .expect("Unable to launch Listner Worker");

        for dir in watch_dirs {
            if let Err(e) = watcher.watch(&dir, RecursiveMode::Recursive) {
                println!("Unable to watch directory {:?}:\n{:#?}", &dir, e);
            }
        }

        let tx = &mut self.listener_tx;
        for event in watcher_rx.iter() {
            println!("Event: {:#?}", event);

            match event {
                DebouncedEvent::Create(path) | DebouncedEvent::Write(path) => {
                    tx.send(ListenerEvent::Exist(path))
                        .expect("Listener send channel closed.");
                }

                DebouncedEvent::Rename(old_path, new_path) => {
                    tx.send(ListenerEvent::Remove(old_path))
                        .expect("Listener send channel closed.");
                    tx.send(ListenerEvent::Exist(new_path))
                        .expect("Listener send channel closed.");
                }

                DebouncedEvent::Remove(path) => {
                    tx.send(ListenerEvent::Remove(path))
                        .expect("Listener send channel closed.");
                }

                DebouncedEvent::Rescan => {
                    tx.send(ListenerEvent::Reload)
                        .expect("Listener send channel closed.");
                }

                DebouncedEvent::Error(error, p) => {
                    if let Some(path) = p {
                        println!(
                            "Underlying filesystem watcher error at path: {:?}\n{:#?}",
                            path, error
                        );
                    } else {
                        println!("Underlying filesystem watcher error: {:#?}", error);
                    }
                    tx.send(ListenerEvent::Reload)
                        .expect("Listener send channel closed.");
                }

                _ => (),
            }
        }
    }
}

#[derive(Debug)]
pub enum ListenerEvent {
    Exist(PathBuf),
    Remove(PathBuf),
    Reload,
}
