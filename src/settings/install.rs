use super::config;
use std::path::PathBuf;
use failure::Error;

pub struct InstallInfo {
    id: Option<String>,
    id_location: PathBuf,
}

impl InstallInfo {
    pub fn new() -> Result<Self, Error> {
        let mut location = config::get_config_dir()?.0;
        location.push("install-id");
        Ok(InstallInfo {
            id: None,
            id_location: location,
        })
    }
}
