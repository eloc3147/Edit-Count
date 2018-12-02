use regex::{Captures, Regex};
use serde::de::{self, Deserialize, Deserializer};
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
    pub fn new(path: Option<&String>, start: usize, length: Option<usize>, message: &str) -> LayoutError {
        LayoutError {
            path: path.map_or(None, |p| Some(p.clone())),
            start,
            length,
            message: String::from(message),
        }
    }
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", "Error loading layout.");
        if let Some(ref path) = self.path {
            writeln!(f, " | {}", path);
        }
        if let Some(length) = self.length {
            writeln!(
                f,
                " | {}{}",
                " ".repeat(self.start),
                "^".repeat(length)
            );
        }
        writeln!(f, "{}", self.message);
        
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

#[derive(Debug, Deserialize)]
pub struct DirectoryLayout {
    #[serde(deserialize_with = "deserialize_dirs")]
    raw_dirs: Vec<DirectoryPath>,
    #[serde(deserialize_with = "deserialize_dirs")]
    render_dirs: Vec<DirectoryPath>,
}

#[derive(Debug)]
pub struct DirectoryPath {
    path: Vec<PathComponent>,
}

#[derive(Debug)]
pub enum PathComponent {
    Album(Album),
    Group(Group),
    Dir(PathBuf),
}

#[derive(Debug)]
pub struct Group {
    depth: Option<usize>,
}

#[derive(Debug)]
pub struct Album {
    min: usize,
    max: Option<usize>,
    tipe: AlbumType,
}

#[derive(Debug)]
enum AlbumType {
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
        parsed.push(parse_path(p).map_err(de::Error::custom)?);
    }

    Ok(parsed)
}

fn parse_path(s: String) -> Result<DirectoryPath, LayoutError> {
    lazy_static! {
        static ref ALBUM_PATTERN: Regex =
            Regex::new(r"\[A(?P<min>\d+)?(?P<dot>.(?P<max>\d+)?)?\]").unwrap();
        static ref GROUP_PATTERN: Regex =
            Regex::new(r"\[G(?P<depth>\d+)?\]").unwrap();
    }

    let mut path = Vec::new();
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

                let max: Option<usize> = match get_key(&captures, "max").parse() {
                    Ok(i) => Some(i),
                    Err(_) => None,
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
                let depth: Option<usize> = match get_key(&captures, "dot").parse() {
                    Ok(i) => Some(i),
                    Err(_) => None,
                };

                component = PathComponent::Group(Group { depth });

            // Invalid operator
            } else {
                return Err(LayoutError::new(
                    Some(&s),
                    index,
                    Some(op_string.len()),
                    "Invalid operator.\nSee README.md for correct operator usage.",
                ));
            }

            path.push(component);
        
        // Path
        } else {
            // Fix windows drive letter path being relative
            let mut path_string = String::from(op_string);
            if path_string.ends_with(':') {
                path_string.push('\\');
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

    Ok(DirectoryPath { path })
}

fn get_key(caps: &Captures, key: &str) -> String {
     match caps.name(key) {
        Some(s) => s.as_str().into(),
        None => "".into(),
    }
}
