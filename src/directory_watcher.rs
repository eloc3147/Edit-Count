pub mod crawler;
pub mod directory_layout;
pub mod listener;

use self::crawler::Crawler;
use self::directory_layout::DirectoryLayout;
use self::listener::{Listener, ListenerEvent};
use std::error::Error;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;

#[derive(Debug)]
pub struct DirectoryWatcher {
    layout: Arc<DirectoryLayout>,
    crawler: Crawler,
    listener: Listener,
    listener_rx: Receiver<ListenerEvent>,
}

impl DirectoryWatcher {
    pub fn launch(
        watcher_frequency: u64,
        layout: Arc<DirectoryLayout>,
    ) -> Result<Self, Box<Error>> {
        let (listener_tx, listener_rx) = channel();
        let listener = Listener::launch(watcher_frequency, layout.clone(), listener_tx)?;
        let crawler = Crawler::launch(layout.clone());

        Ok(DirectoryWatcher {
            layout,
            crawler,
            listener,
            listener_rx,
        })
    }

    pub fn join(self) {
        while let Ok(event) = self.listener_rx.recv() {
            println!("===  {:#?}", event);
        }
        self.crawler.join();
        self.listener.join();
    }
}
