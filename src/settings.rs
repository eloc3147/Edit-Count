use super::directory_watcher::directory_layout::DirectoryLayout;
use config::{Config, ConfigError, File};
use serde_derive::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub watch_frequency: u64,
    pub directory_layout: DirectoryLayout,
}

impl Settings {
    pub fn new(config_dir: &PathBuf) -> Result<Self, ConfigError> {
        let mut s = Config::new();

        s.merge(File::from(config_dir.join("settings.toml")))?;

        s.try_into()
    }
}
