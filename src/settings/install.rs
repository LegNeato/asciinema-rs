use super::config;
use std::path::PathBuf;
use failure::Error;
use uuid::Uuid;
use std::fs::File;
use std::fs::create_dir_all;
use std::io::Read;
use std::io::Write;

#[derive(Clone, Debug)]
pub struct InstallInfo {
    pub id: Option<Uuid>,
    pub location: PathBuf,
}

impl InstallInfo {
    pub fn new() -> Result<Self, Error> {
        let mut location = config::get_config_dir()?.0;
        location.push("install-id");
        if location.exists() {
            let mut f = File::open(&location).expect("file not found");

            let mut contents = String::new();
            f.read_to_string(&mut contents)
                .expect("something went wrong reading the file");

            let parsed_id = Uuid::parse_str(contents.trim()).unwrap();

            Ok(InstallInfo {
                id: Some(parsed_id),
                location,
            })
        } else {
            Ok(InstallInfo { id: None, location })
        }
    }

    pub fn generate(self) -> Self {
        InstallInfo {
            id: Some(Uuid::new_v4()),
            location: self.location,
        }
    }

    pub fn save(self) -> Result<(), Error> {
        create_dir_all(self.location.parent().unwrap())?;
        // Write the file.
        let mut f = File::create(&self.location).expect("Unable to create file");
        f.write_all(self.id.unwrap().hyphenated().to_string().as_bytes())
            .expect("Unable to write data");
        Ok(())
    }
}
