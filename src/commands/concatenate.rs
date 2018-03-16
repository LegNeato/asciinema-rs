use asciicast::Entry;
use failure::Error;
use reqwest::{self, StatusCode};
use settings::ConcatenateSettings;
use std::fs::File;
use std::io::copy;
use std::io::prelude::*;
use std::io::{self, BufReader, Write};
use serde_json;
use tempfile::NamedTempFile;

#[derive(Debug, Fail)]
enum ConcatenateFailure {
    #[fail(display = "target resource not found: {}", res)]
    NotFound { res: String },
    #[fail(display = "something else happened (status: {})", stat)]
    Others { stat: String },
}

pub fn go(settings: &ConcatenateSettings) -> Result<(), Error> {
    let location = settings.location.clone();

    let mut temp: NamedTempFile = NamedTempFile::new()?;

    // This looks silly.
    let file: File;

    let if_url = location.to_str().unwrap().starts_with("http");
    if if_url {
        let l = location.with_extension("cast");
        let target = l.to_str().unwrap();
        let mut response = reqwest::get(target)?;
        match response.status() {
            StatusCode::Ok => {}
            StatusCode::NotFound => Err(ConcatenateFailure::NotFound {
                res: target.to_string(),
            })?,
            s => Err(ConcatenateFailure::Others {
                stat: s.to_string(),
            })?,
        };
        copy(&mut response, &mut temp).unwrap();
        file = temp.reopen()?;
    } else {
        file = File::open(location)?;
    }

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    let mut reader = BufReader::new(file);
    let mut line = String::new();

    // Skip the first line, and maybe Header is needed later.
    let _len = reader.read_line(&mut line);

    for line in reader.lines() {
        let entry: Entry = serde_json::from_str(line.unwrap().as_str())?;
        handle.write(entry.event_data.as_bytes())?;
    }

    Ok(())
}
