use super::directory_layout::{AlbumType, DirectoryLayout, PathComponent};
use super::{DirectoryUpdateEvent, GroupType, SetEvent};
use crate::worker::{Worker, WorkerResult};
use derive_new::new;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

#[derive(Debug, new)]
pub struct Crawler {
    layout: DirectoryLayout,
    due_tx: Sender<DirectoryUpdateEvent>,
}

impl Worker for Crawler {
    type W = Crawler;
    const NAME: &'static str = "Crawler";

    fn work(self) -> WorkerResult {
        let mut paths = Vec::new();
        for path in self.layout.raw_dirs.iter() {
            paths.push((GroupType::Raw, path));
        }
        for path in self.layout.render_dirs.iter() {
            paths.push((GroupType::Render, path));
        }

        for (path_type, path) in paths {
            let mut path_cache = PathBuf::new();
            let mut groups: Vec<NamedPath> = Vec::new();
            let mut filled_groups: Vec<(NamedPath, Vec<NamedPath>)> = Vec::new();
            for component in path {
                match component {
                    PathComponent::Group(group) => {
                        // Process group

                        let mut new_groups: Vec<NamedPath> = Vec::new();
                        if groups.is_empty() {
                            for entry in deep_list(&path_cache, group.depth - 1, group.depth) {
                                new_groups.push(NamedPath {
                                    name: entry.to_string_lossy().into_owned(),
                                    path: path_cache.join(entry),
                                });
                            }
                        } else {
                            for old_group in groups {
                                let search_path = old_group.path.join(&path_cache);
                                for entry in deep_list(&search_path, group.depth - 1, group.depth) {
                                    new_groups.push(NamedPath {
                                        name: format!(
                                            "{}:{}",
                                            old_group.name,
                                            entry.to_string_lossy().into_owned(),
                                        ),
                                        path: search_path.join(entry),
                                    });
                                }
                            }
                        }

                        groups = new_groups;
                        path_cache = PathBuf::new();
                    }

                    PathComponent::Album(album) => {
                        // Process album

                        // If the groups Vec is empty, then no groups were found.
                        // Put all albums into a virtual group that will be hidden in the web view.
                        if groups.is_empty() {
                            groups = vec![NamedPath {
                                name: String::from("%default%"),
                                path: PathBuf::new(),
                            }];
                        }

                        for group in groups {
                            // Determine search depth
                            let (filter_depth, search_depth) = match &album.tipe {
                                AlbumType::Single => (0, 1),
                                AlbumType::Depth => (album.min - 1, album.min),
                                AlbumType::Range => (album.min - 1, album.max),
                            };

                            let group_path = group.path.join(&path_cache);
                            let entries = deep_list(&group_path, filter_depth, search_depth);
                            let mut album_cache = Vec::with_capacity(entries.len());

                            for entry in entries {
                                album_cache.push(NamedPath {
                                    name: entry.to_string_lossy().into_owned(),
                                    path: group_path.join(entry),
                                });
                            }

                            filled_groups.push((group, album_cache));
                        }

                        groups = Vec::new();
                        path_cache = PathBuf::new();
                    }

                    PathComponent::Dir(dir) => {
                        // Process path

                        path_cache.push(dir);
                    }
                }
            }

            if !path_cache.as_os_str().is_empty() {
                for group in &mut filled_groups {
                    for album in &mut group.1 {
                        album.path.push(&path_cache);
                    }
                }
            }

            for (group, albums) in filled_groups {
                for NamedPath { name, path } in albums {
                    let mut files = Vec::new();
                    let contents = path.read_dir();

                    if contents.is_err() {
                        println!("Error reading folder {:?}: {:#?}", path, contents);
                    }

                    for file in contents.unwrap() {
                        match file {
                            Ok(f) => files.push(f.file_name()),
                            Err(e) => println!("Error reading file: {:#?}", e),
                        }
                    }

                    let event = SetEvent {
                        group_name: group.name.to_string(),
                        album_name: name,
                        tipe: path_type,
                        files,
                    };

                    self.due_tx.send(DirectoryUpdateEvent::Set(event))?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct NamedPath {
    pub name: String,
    pub path: PathBuf,
}

fn deep_list(base: &PathBuf, filter_depth: usize, search_depth: usize) -> Vec<PathBuf> {
    let mut final_dirs = Vec::new();
    let mut search_paths: Vec<PathBuf> = vec![base.into()];

    for i in 0..search_depth {
        let mut new_search_paths: Vec<PathBuf> = Vec::new();
        for path in search_paths {
            // The loop is bootstrapped with the base path.
            // All other entries in new_dirs will be relative and must be joined to the base path.
            let dirs = if i == 0 {
                path.read_dir()
            } else {
                base.join(&path).read_dir()
            };

            // Warn and skip if listing failed
            let dirs = match dirs {
                Ok(d) => d,
                Err(e) => {
                    println!("{:?}", e);
                    continue;
                }
            };

            for entry in dirs {
                // Warn and skip errors
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(e) => {
                        println!("{:?}", e);
                        continue;
                    }
                };

                let file_type = match entry.file_type() {
                    Ok(file_type) => file_type,
                    Err(e) => {
                        println!("Couldn't get file type for {:?}: {:#?}", entry.path(), e);
                        continue;
                    }
                };

                // Ignore files
                if file_type.is_dir() {
                    let name: PathBuf;

                    // Don't include base path
                    if i == 0 {
                        name = entry.file_name().into();
                    } else {
                        name = path.join(entry.file_name())
                    }
                    new_search_paths.push(name);
                }
            }
        }

        if new_search_paths.is_empty() {
            break;
        }

        if i >= filter_depth {
            for path in new_search_paths.iter() {
                final_dirs.push(path.clone());
            }
        }

        search_paths = new_search_paths;
    }

    final_dirs
}
