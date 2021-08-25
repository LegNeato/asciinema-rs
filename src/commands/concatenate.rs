use crate::settings::ConcatenateSettings;
use asciicast::{Entry, Header};
use failure::{Error, Fail};
use reqwest::{self, StatusCode};
use serde_json;
use std::fs::File;
use std::io::copy;
use std::io::prelude::*;
use std::io::{self, BufReader, Write};
use std::path::PathBuf;
use tempfile::NamedTempFile;

#[derive(Debug, Fail)]
enum ConcatenateFailure {
    #[fail(display = "header not found")]
    HeaderNotFound,
}

#[derive(Debug, Fail)]
enum ReqwestFailure {
    #[fail(display = "target resource not found: {}", res)]
    NotFound { res: String },
    #[fail(display = "something else happened (status: {})", stat)]
    Others { stat: String },
}

pub fn get_file(location: PathBuf, temp: &mut NamedTempFile) -> Result<File, Error> {
    let file: File;

    let if_url = location.to_str().unwrap().starts_with("http");
    if if_url {
        let l = location.with_extension("cast");
        let target = l.to_str().unwrap();
        let mut response = reqwest::get(target)?;
        match response.status() {
            StatusCode::OK => {}
            StatusCode::NOT_FOUND => {
                return Err(ReqwestFailure::NotFound {
                    res: target.to_string(),
                }
                .into())
            }
            s => {
                return Err(ReqwestFailure::Others {
                    stat: s.to_string(),
                }
                .into())
            }
        };
        copy(&mut response, temp).unwrap();
        file = temp.reopen()?;
    } else {
        file = File::open(location)?;
    }
    Ok(file)
}

pub fn go(settings: &ConcatenateSettings) -> Result<(), Error> {
    let location = settings.location.clone();

    let mut temp: NamedTempFile = NamedTempFile::new()?;

    let file = get_file(location, &mut temp)?;

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    let mut reader = BufReader::new(file);
    let mut line = String::new();

    // Skip the first line, and maybe Header is needed later.
    let _len = reader.read_line(&mut line);
    let res: Result<Header, serde_json::Error> = serde_json::from_str(line.as_str());
    let _header = match res {
        Ok(h) => h,
        Err(_) => return Err(ConcatenateFailure::HeaderNotFound.into()),
    };

    for line in reader.lines() {
        let entry: Entry = serde_json::from_str(line.unwrap().as_str())?;
        handle.write_all(entry.event_data.as_bytes())?;
    }

    Ok(())
}
