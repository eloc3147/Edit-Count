use lazy_static::lazy_static;
use regex::{Captures, Regex};
use serde::de::{self, Deserialize, Deserializer};
use serde_derive::Deserialize;
use std::error;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub struct LayoutError {
    path: Option<String>,
    start: usize,
    length: Option<usize>,
    message: String,
}

impl LayoutError {
    pub fn new(
        path: Option<&str>,
        start: usize,
        length: Option<usize>,
        message: &str,
    ) -> LayoutError {
        LayoutError {
            path: path.and_then(|p| Some(p.to_owned())),
            start,
            length,
            message: String::from(message),
        }
    }
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Error loading layout.").unwrap();
        if let Some(ref path) = self.path {
            writeln!(f, " | {}", path).unwrap();
        }
        if let Some(length) = self.length {
            writeln!(f, " | {}{}", " ".repeat(self.start), "^".repeat(length)).unwrap();
        }
        writeln!(f, "{}", self.message).unwrap();

        Ok(())
    }
}

impl error::Error for LayoutError {
    fn description(&self) -> &str {
        "unable to parse layout"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DirectoryLayout {
    #[serde(deserialize_with = "deserialize_dirs")]
    pub raw_dirs: Vec<DirectoryPath>,
    #[serde(deserialize_with = "deserialize_dirs")]
    pub render_dirs: Vec<DirectoryPath>,
}

pub type DirectoryPath = Vec<PathComponent>;

#[derive(Debug, Clone)]
pub enum PathComponent {
    Album(Album),
    Group(Group),
    Dir(PathBuf),
}

#[derive(Debug, Clone)]
pub struct Group {
    pub depth: usize,
}

#[derive(Debug, Clone)]
pub struct Album {
    pub min: usize,
    pub max: usize,
    pub tipe: AlbumType,
}

#[derive(Debug, Clone)]
pub enum AlbumType {
    Single,
    Depth,
    Range,
}

fn deserialize_dirs<'de, D>(deserializer: D) -> Result<Vec<DirectoryPath>, D::Error>
where
    D: Deserializer<'de>,
{
    let unparsed: Vec<String> = Deserialize::deserialize(deserializer)?;
    let mut parsed: Vec<DirectoryPath> = Vec::with_capacity(unparsed.len());
    for p in unparsed {
        parsed.push(parse_path(&p).map_err(de::Error::custom)?);
    }

    Ok(parsed)
}

fn parse_path(s: &str) -> Result<DirectoryPath, LayoutError> {
    lazy_static! {
        static ref ALBUM_PATTERN: Regex =
            Regex::new(r"\[A(?P<min>\d+)?(?P<dot>.(?P<max>\d+)?)?\]").unwrap();
        static ref GROUP_PATTERN: Regex = Regex::new(r"\[G(?P<depth>\d+)?\]").unwrap();
    }

    let mut path = DirectoryPath::new();
    let mut path_cache = PathBuf::new();
    let mut cache_empty = true;
    let mut index = 0;
    let mut album = 0;

    for op_string in s.split(|c| c == '\\' || c == '/') {
        let component: PathComponent;

        // Operator
        if op_string.starts_with('[') && op_string.ends_with(']') {
            // Dump path cache
            if !cache_empty {
                path.push(PathComponent::Dir(path_cache));
                path_cache = PathBuf::new();
                cache_empty = true;
            }

            // Album operator
            if let Some(captures) = ALBUM_PATTERN.captures(op_string) {
                let min: usize = match get_key(&captures, "min").parse() {
                    Ok(i) => i,
                    Err(_) => 1,
                };

                let max: usize = match get_key(&captures, "max").parse() {
                    Ok(i) => i,
                    Err(_) => usize::max_value(),
                };

                let tipe: AlbumType;
                if get_key(&captures, "dot") == "." {
                    tipe = AlbumType::Range;
                } else if min != 1 {
                    tipe = AlbumType::Depth;
                } else {
                    tipe = AlbumType::Single;
                }

                component = PathComponent::Album(Album { min, max, tipe });
                album += 1;

            // Group operator
            } else if let Some(captures) = GROUP_PATTERN.captures(op_string) {
                let depth: usize = match get_key(&captures, "depth").parse() {
                    Ok(i) => i,
                    Err(_) => 1,
                };

                component = PathComponent::Group(Group { depth });

            // Invalid operator
            } else {
                return Err(LayoutError::new(
                    Some(s),
                    index,
                    Some(op_string.len()),
                    "Invalid operator.\nSee README.md for correct operator usage.",
                ));
            }

            path.push(component);

        // Path
        } else {
            let mut path_string = String::from(op_string);

            // Make paths absolute
            if path.is_empty() && cache_empty {
                if cfg!(windows) {
                    path_string += "\\";
                } else {
                    path_string = format!("/{}", path_string);
                }
            }
            path_cache.push(path_string);
            cache_empty = false;
        }
        index += op_string.len() + 1;
    }

    if album > 1 {
        return Err(LayoutError::new(
            Some(&s),
            0,
            None,
            "Missing Album operator.",
        ));
    } else if album < 1 {
        return Err(LayoutError::new(
            Some(&s),
            0,
            None,
            "Multiple Album operators.\nOnly one Album operator is allowed.",
        ));
    }

    Ok(path)
}

fn get_key(caps: &Captures, key: &str) -> String {
    match caps.name(key) {
        Some(s) => s.as_str().into(),
        None => "".into(),
    }
}
