use super::{Count, CountUpdateEvent, DirectoryUpdateEvent, GroupType, SetEvent};
use crate::worker::Worker;
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};

pub struct Counter {
    due_rx: Receiver<DirectoryUpdateEvent>,
    cue_tx: Sender<CountUpdateEvent>,
    tree: Arc<RwLock<CountTree>>,
}

#[derive(Debug)]
pub struct CountTree {
    counts: HashMap<String, Group>,
    totals: Totals,
    nonce: usize,
}

#[derive(Debug)]
struct Totals {
    totals: HashMap<String, HashSet<OsString>>,
}

type Group = HashMap<String, Album>;

#[derive(Debug)]
struct Album {
    raw_cache: HashSet<OsString>,
    render_cache: HashSet<OsString>,
}

impl Counter {
    pub fn new(
        due_rx: Receiver<DirectoryUpdateEvent>,
        cue_tx: Sender<CountUpdateEvent>,
    ) -> Counter {
        Counter {
            due_rx,
            cue_tx,
            tree: Arc::new(RwLock::new(CountTree::new())),
        }
    }

    pub fn get_tree_reference(&self) -> Arc<RwLock<CountTree>> {
        self.tree.clone()
    }
}

impl Worker for Counter {
    type W = Counter;
    const NAME: &'static str = "Counter";

    fn work(self) {
        while let Ok(event) = self.due_rx.recv() {
            let cue = match event {
                DirectoryUpdateEvent::Exist(path) => {
                    unimplemented!();
                }
                DirectoryUpdateEvent::Remove(path) => {
                    unimplemented!();
                }
                DirectoryUpdateEvent::Set(event) => self.tree.write().unwrap().set(event),
            };
            if cue.count.total > 0 {
                self.cue_tx.send(cue).unwrap();
            }
        }
    }
}

impl CountTree {
    pub fn new() -> CountTree {
        CountTree {
            counts: HashMap::new(),
            totals: Totals::new(),
            nonce: 0,
        }
    }

    pub fn set(&mut self, event: SetEvent) -> CountUpdateEvent {
        let mut raw_count = 0;
        let mut render_count = 0;

        // Ensure only raw files are added to the total counts
        let total_count = match event.tipe {
            GroupType::Raw => {
                self.totals
                    .update_count(&event.album_name, &event.group_name, &event.files)
            }
            GroupType::Render => {
                self.totals
                    .update_count(&event.album_name, &event.group_name, &Vec::new())
            }
        };

        // Convert files to HashSet
        let mut file_set = HashSet::with_capacity(event.files.len());
        for file in event.files {
            file_set.insert(file);
        }

        let group = self.get_group(&event.group_name);

        match group.get_mut(&event.album_name) {
            // If the album already exists
            Some(album) => {
                // Update the existing album
                match event.tipe {
                    GroupType::Raw => {
                        album.raw_cache = file_set;
                    }
                    GroupType::Render => {
                        album.render_cache = file_set;
                    }
                }
                raw_count = album.raw_cache.len();
                render_count = album.render_cache.len();
            }

            // If the album doesn't already exist
            None => {
                // Create a new album
                let (raw_cache, render_cache) = match event.tipe {
                    GroupType::Raw => {
                        raw_count = file_set.len();
                        (file_set, HashSet::new())
                    }
                    GroupType::Render => {
                        render_count = file_set.len();
                        (HashSet::new(), file_set)
                    }
                };

                let album = Album {
                    raw_cache,
                    render_cache,
                };

                // Add the new album
                group.insert(event.album_name.clone(), album);
            }
        }

        CountUpdateEvent {
            group_name: event.group_name,
            album_name: event.album_name,
            count: Count {
                total: total_count,
                raw: raw_count,
                render: render_count,
            },
        }
    }

    fn get_group(&mut self, name: &str) -> &mut Group {
        if !self.counts.contains_key(name) {
            self.counts.insert(name.to_string(), Group::new());
        }

        self.counts.get_mut(name).unwrap()
    }
}

impl Totals {
    pub fn new() -> Totals {
        Totals {
            totals: HashMap::new(),
        }
    }

    pub fn update_count(
        &mut self,
        album_name: &str,
        group_name: &str,
        names: &[OsString],
    ) -> usize {
        let key = format!("{}\n{}", album_name, group_name);
        let album = match self.totals.get_mut(&key) {
            Some(a) => a,
            None => {
                self.totals
                    .insert(key.clone(), HashSet::with_capacity(names.len()));
                self.totals.get_mut(&key).unwrap()
            }
        };

        for name in names {
            // HashSet::insert will not update the set if the key already exists
            // This should be less expensive than checking if the value exists first
            album.insert(name.clone());
        }

        album.len()
    }
}
