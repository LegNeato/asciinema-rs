use super::config;
use failure::Error;
use std::fs::File;
use std::fs::create_dir_all;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;

fn get_install_id_file() -> Result<PathBuf, Error> {
    let mut location = config::get_config_dir()?.0;
    location.push("install-id");
    Ok(location)
}

#[derive(Clone, Debug, Deserialize)]
pub struct InstallInfo {
    pub id: Uuid,
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

            Ok(InstallInfo { id, location })
        } else {
            // The reference python client always saves.
            let info = InstallInfo {
                id: Uuid::new_v4(),
                location,
            };
            info.save()?;
            Ok(info)
        }
    }

    pub fn save(&self) -> Result<(), Error> {
        create_dir_all(self.location.parent().unwrap())?;
        // Write the file.
        let mut f = File::create(&self.location)?;
        f.write_all(self.id.hyphenated().to_string().as_bytes())?;
        Ok(())
    }
}

impl FromStr for InstallInfo {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = Uuid::parse_str(s.trim())?;
        let location = get_install_id_file()?;

        let info = InstallInfo { id, location };
        info.save()?;
        Ok(info)
    }
}
