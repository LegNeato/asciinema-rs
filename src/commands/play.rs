use asciicast::{Entry, Header};
use commands::record::get_elapsed_seconds;
use commands::concatenate::get_file;
use failure::Error;
use serde_json;
use settings::PlaySettings;
use std::io::prelude::*;
use std::io::{self, BufReader, Write};
use std::time::Instant;
use tempfile::NamedTempFile;
use termion;

#[derive(Debug, Fail)]
enum PlayFailure {
    #[fail(display = "header not found")]
    HeaderNotFound,
}

pub fn go(settings: &PlaySettings) -> Result<(), Error> {
    let location = settings.location.clone();

    let mut temp: NamedTempFile = NamedTempFile::new()?;

    let file = get_file(location, &mut temp)?;

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    // Hide the cursor if requested to do so.
    if settings.hide_cursor {
        handle.write_all(format!("{}", termion::cursor::Hide).as_bytes())?;
        handle.flush()?;
    }

    let mut reader = BufReader::new(file);
    let mut line = String::new();

    // Skip the first line, and maybe Header is needed later.
    let _len = reader.read_line(&mut line);
    let res: Result<Header, serde_json::Error> = serde_json::from_str(line.as_str());
    let _header = match res {
        Ok(h) => h,
        Err(_) => return Err(PlayFailure::HeaderNotFound)?,
    };

    let base = Instant::now();

    let speed_factor = match settings.speed {
        Some(s) => 1.0 / s,
        None => 1.0,
    };

    for line in reader.lines() {
        let entry: Entry = serde_json::from_str(line.unwrap().as_str())?;
        loop {
            if entry.time * speed_factor <= get_elapsed_seconds(&base.elapsed()) {
                handle.write_all(entry.event_data.as_bytes())?;
                handle.flush()?;
                break;
            }
        }
    }

    // Restore the cursor if it was previously hidden.
    if settings.hide_cursor {
        handle.write_all(format!("{}", termion::cursor::Show).as_bytes())?;
        handle.flush()?;
    }

    Ok(())
}
