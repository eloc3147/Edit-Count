use crate::worker::{Worker, WorkerError, WorkerResult};
use crate::{Count, CountUpdateEvent, DirectoryUpdateEvent, GroupType, SetEvent};
use derive_new::new;
use failure::Error;
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

#[derive(new)]
pub struct Counter {
    cue_tx: Sender<CountUpdateEvent>,
    due_rx: Receiver<DirectoryUpdateEvent>,
    #[new(default)]
    tree: Arc<Mutex<CountTree>>,
}

impl Counter {
    pub fn get_handle(&self) -> CounterHandle {
        CounterHandle(self.tree.clone())
    }
}

impl Worker for Counter {
    type W = Counter;
    const NAME: &'static str = "Counter";

    fn work(self) -> WorkerResult {
        while let Ok(event) = self.due_rx.recv() {
            let mut tree = self.tree.lock().or(Err(WorkerError::ResourcePoisoned {
                name: "Counter.tree".to_string(),
            }))?;

            match event {
                DirectoryUpdateEvent::Exist(path) => {
                    // TODO: Implement
                    println!("Exist!({:?})", path)
                }

                DirectoryUpdateEvent::Remove(path) => {
                    // TODO: Implement
                    println!("Remove!({:?})", path)
                }

                DirectoryUpdateEvent::Set(event) => {
                    let cue = tree.set(event)?;
                    if cue.count.total > 0 {
                        self.cue_tx.send(cue)?;
                    }
                }

                DirectoryUpdateEvent::Refresh => {
                    for cue in tree.full_count()? {
                        self.cue_tx.send(cue)?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct CounterHandle(Arc<Mutex<CountTree>>);

impl CounterHandle {
    pub fn full_count(&self) -> Result<Vec<CountUpdateEvent>, Error> {
        self.0
            .lock()
            .or(Err(WorkerError::new_resource_poisoned(
                "Counter.tree".to_string(),
            )))?
            .full_count()
    }
}

#[derive(Debug, Default, new, Clone)]
pub struct CountTree {
    #[new(default)]
    counts: HashMap<String, Group>,
    #[new(default)]
    totals: Totals,
    #[new(default)]
    nonce: usize,
}

impl CountTree {
    pub fn full_count(&self) -> Result<Vec<CountUpdateEvent>, Error> {
        let mut counts = Vec::new();
        for (group_name, group) in self.counts.iter() {
            for (album_name, album) in group.iter() {
                let raw_count = album.raw_cache.len();
                let render_count = album.render_cache.len();
                let total_count = self.totals.get_count(&album_name, &group_name)?;

                let cue = CountUpdateEvent {
                    group_name: group_name.clone(),
                    album_name: album_name.clone(),
                    count: Count {
                        total: total_count,
                        raw: raw_count,
                        render: render_count,
                    },
                };

                counts.push(cue);
            }
        }

        Ok(counts)
    }

    pub fn set(&mut self, event: SetEvent) -> Result<CountUpdateEvent, Error> {
        let mut raw_count = 0;
        let mut render_count = 0;

        // Ensure only raw files are added to the total counts
        if event.tipe == GroupType::Raw {
            self.totals
                .update_count(&event.album_name, &event.group_name, &event.files)?;
        }
        let total_count = self
            .totals
            .get_count(&event.album_name, &event.group_name)?;

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

        Ok(CountUpdateEvent {
            group_name: event.group_name,
            album_name: event.album_name,
            count: Count {
                total: total_count,
                raw: raw_count,
                render: render_count,
            },
        })
    }

    fn get_group(&mut self, name: &str) -> &mut Group {
        if !self.counts.contains_key(name) {
            self.counts.insert(name.to_string(), Group::new());
        }

        self.counts.get_mut(name).expect(
            format!(
                "CountTree.counts does not contain {}, this should not happen.",
                name
            )
            .as_str(),
        )
    }
}

#[derive(Debug, Default, new, Clone)]
struct Totals(HashMap<String, HashSet<OsString>>);

impl Totals {
    pub fn update_count(
        &mut self,
        album_name: &str,
        group_name: &str,
        names: &[OsString],
    ) -> WorkerResult {
        let key = format!("{}\n{}", album_name, group_name);

        let album = match self.0.get_mut(&key) {
            Some(a) => a,
            None => {
                self.0
                    .insert(key.clone(), HashSet::with_capacity(names.len()));
                self.0.get_mut(&key).expect(
                    format!(
                        "Totals.totals does not contain {}, this should not happen.",
                        key
                    )
                    .as_str(),
                )
            }
        };

        for name in names {
            // HashSet::insert will not update the set if the key already exists
            // This should be less expensive than checking if the value exists first
            album.insert(name.clone());
        }

        Ok(())
    }

    pub fn get_count(&self, album_name: &str, group_name: &str) -> Result<usize, Error> {
        let key = format!("{}\n{}", album_name, group_name);
        match self.0.get(&key) {
            Some(a) => Ok(a.len()),
            None => Ok(0),
        }
    }
}

type Group = HashMap<String, Album>;

#[derive(Debug, Clone)]
struct Album {
    raw_cache: HashSet<OsString>,
    render_cache: HashSet<OsString>,
}
