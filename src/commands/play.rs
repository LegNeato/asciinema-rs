use asciicast::{Entry, Header};
use commands::concatenate::get_file;
use commands::record::get_elapsed_seconds;
use failure::Error;
use serde_json;
use settings::PlaySettings;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader, StdoutLock, Write};
use std::time::Instant;
use tempfile::NamedTempFile;
use termion;

#[derive(Debug, Fail)]
enum PlayFailure {
    #[fail(display = "header not found")]
    HeaderNotFound,
}

fn write_with_time_limit(
    handle: &mut StdoutLock,
    reader: BufReader<File>,
    idle_time_limit: Option<f64>,
    speed_factor: f64,
) -> Result<(), Error> {
    let mut t = 0.0_f64;
    let mut last = 0.0_f64;
    let limit = idle_time_limit.unwrap();
    let base = Instant::now();

    for line in reader.lines() {
        let entry: Entry = serde_json::from_str(line.unwrap().as_str())?;
        let delay = entry.time - last;
        last = entry.time;
        t = t + limit.min(delay);

        loop {
            if t * speed_factor <= get_elapsed_seconds(&base.elapsed()) {
                handle.write_all(entry.event_data.as_bytes())?;
                handle.flush()?;
                break;
            }
        }
    }
    Ok(())
}

fn write_without_time_limit(
    handle: &mut StdoutLock,
    reader: BufReader<File>,
    speed_factor: f64,
) -> Result<(), Error> {
    let base = Instant::now();
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
    Ok(())
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
    let header = match res {
        Ok(h) => h,
        Err(_) => return Err(PlayFailure::HeaderNotFound)?,
    };

    let idle_time_limit = if settings.idle_time_limit.is_some() {
        settings.idle_time_limit
    } else if header.idle_time_limit.is_some() {
        header.idle_time_limit
    } else {
        None
    };

    let speed_factor = match settings.speed {
        Some(s) => 1.0 / s,
        None => 1.0,
    };

    if idle_time_limit.is_some() {
        write_with_time_limit(&mut handle, reader, idle_time_limit, speed_factor)?;
    } else {
        write_without_time_limit(&mut handle, reader, speed_factor)?;
    }

    // Restore the cursor if it was previously hidden.
    if settings.hide_cursor {
        handle.write_all(format!("{}", termion::cursor::Show).as_bytes())?;
        handle.flush()?;
    }

    Ok(())
}
