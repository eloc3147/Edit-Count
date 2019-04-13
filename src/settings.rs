use crate::directory_layout::DirectoryLayout;
use config::{Config, ConfigError, File};
use serde_derive::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub watch_frequency: u64,
    pub web_port: u16,
    pub ws_port: u16,
    pub directory_layout: DirectoryLayout,
}

impl Settings {
    pub fn from(config_file: PathBuf) -> Result<Self, ConfigError> {
        let mut s = Config::new();

        s.merge(File::from(config_file))?;

        s.try_into()
    }
}
