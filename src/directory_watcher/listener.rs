use super::directory_layout::{DirectoryLayout, PathComponent};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct Listener {
    handle: thread::JoinHandle<()>,
}

impl fmt::Debug for Listener {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Listener {{ handle: {:?}}}", self.handle)
    }
}

impl Listener {
    pub fn launch(
        watch_frequency: u64,
        layout: Arc<DirectoryLayout>,
        listener_tx: Sender<ListenerEvent>,
    ) -> Result<Listener, Box<Error>> {
        // Build list of all watched base paths
        let mut watch_dirs = Vec::with_capacity(layout.raw_dirs.len() + layout.render_dirs.len());

        for path in layout.raw_dirs.iter().chain(layout.render_dirs.iter()) {
            let mut path_cache = PathBuf::new();
            for component in path {
                match component {
                    PathComponent::Dir(dir) => path_cache.push(dir),
                    _ => break,
                }
            }
            watch_dirs.push(path_cache);
        }

        let handle = thread::spawn(move || {
            ListnerWorker::launch(watch_frequency, watch_dirs, listener_tx);
        });

        Ok(Listener { handle })
    }

    pub fn join(self) {
        self.handle.join().unwrap();
    }
}

struct ListnerWorker;

impl ListnerWorker {
    pub fn launch(
        watch_frequency: u64,
        watch_dirs: Vec<PathBuf>,
        worker_tx: Sender<ListenerEvent>,
    ) {
        let (watcher_tx, watcher_rx) = channel();
        let mut watcher = watcher(watcher_tx, Duration::from_millis(watch_frequency))
            .expect("Unable to launch Listner Worker");

        for dir in watch_dirs {
            if let Err(e) = watcher.watch(&dir, RecursiveMode::Recursive) {
                println!("Unable to watch directory {:?}:\n{:#?}", &dir, e);
            }
        }

        for event in watcher_rx.iter() {
            println!("Event: {:#?}", event);

            match event {
                DebouncedEvent::Create(path)
                | DebouncedEvent::Write(path)
                | DebouncedEvent::Rename(_, path) => {
                    worker_tx
                        .send(ListenerEvent::Exist(path))
                        .expect("Listener send channel closed.");
                }

                DebouncedEvent::Remove(path) => {
                    worker_tx
                        .send(ListenerEvent::Remove(path))
                        .expect("Listener send channel closed.");
                }

                DebouncedEvent::Rescan => {
                    worker_tx
                        .send(ListenerEvent::Reload)
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
                    worker_tx
                        .send(ListenerEvent::Reload)
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
