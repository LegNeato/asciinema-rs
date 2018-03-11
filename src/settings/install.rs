use super::config;
use std::path::PathBuf;
use failure::Error;
use uuid::Uuid;
use std::str::FromStr;
use std::fs::File;
use std::fs::create_dir_all;
use std::io::Read;
use std::io::Write;

fn get_install_id_file() -> Result<PathBuf, Error> {
    let mut location = config::get_config_dir()?.0;
    location.push("install-id");
    Ok(location)
}

#[derive(Clone, Debug, Deserialize)]
pub struct InstallInfo {
    pub id: Uuid,
    pub is_saved: bool,
    pub location: PathBuf,
}

impl InstallInfo {
    pub fn new() -> Result<Self, Error> {
        let location = get_install_id_file()?;

        if location.exists() {
            let mut f = File::open(&location)?;

            let mut contents = String::new();
            f.read_to_string(&mut contents)?;

            let id = Uuid::parse_str(contents.trim())?;

            Ok(InstallInfo {
                id,
                is_saved: true,
                location,
            })
        } else {
            Ok(InstallInfo {
                id: Uuid::new_v4(),
                is_saved: false,
                location,
            })
        }
    }

    pub fn save(mut self) -> Result<(), Error> {
        create_dir_all(self.location.parent().unwrap())?;
        // Write the file.
        let mut f = File::create(&self.location)?;
        f.write_all(self.id.hyphenated().to_string().as_bytes())?;
        self.is_saved = true;
        Ok(())
    }
}

impl FromStr for InstallInfo {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = Uuid::parse_str(s.trim())?;
        let location = get_install_id_file()?;

        Ok(InstallInfo {
            id,
            is_saved: false,
            location,
        })
    }
}
