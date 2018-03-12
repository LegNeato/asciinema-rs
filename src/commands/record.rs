extern crate asciicast;
extern crate chrono;
extern crate libc;
extern crate pty_shell;
extern crate termcolor;
extern crate url;

extern crate serde_json;

use std::io::prelude::*;
use chrono::Utc;
use std::env;
use std::str;
use uploader::UploadBuilder;

use std::io::LineWriter;
use pty_shell::*;
use std::time::Instant;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::path::PathBuf;
use std::result::Result;
use failure::{err_msg, Error};
use failure::ResultExt;
use url::Url;
use termion;
use tempfile::NamedTempFile;

use settings::RecordSettings;

#[derive(Debug, Fail)]
enum RecordFailure {
    #[fail(display = "unable to write to file: {}: file exists", path)] FileExists { path: String },
}

pub enum RecordLocation {
    Local(PathBuf),
    Remote(Url),
}

struct Shell<W: ?Sized>
where
    W: Write,
{
    writer: LineWriter<Box<W>>,
    clock: Instant,
}

impl<W: ?Sized> PtyHandler for Shell<W>
where
    W: Write,
{
    fn input(&mut self, _input: &[u8]) {
        /* do something with input */
        //println!("In: {:?}", input);
    }

    fn output(&mut self, output: &[u8]) {
        let elapsed = self.clock.elapsed();
        let elapsed_seconds: f64 =
            elapsed.as_secs() as f64 + (0.000_000_001 * f64::from(elapsed.subsec_nanos()));

        let entry = asciicast::Entry {
            time: elapsed_seconds,
            event_type: asciicast::EventType::Output,
            event_data: str::from_utf8(output).unwrap().to_string(),
        };

        // Serialize it to a JSON string.
        let j = serde_json::to_string(&entry).unwrap();
        if let Err(e) = writeln!(self.writer, "{}", j) {
            eprintln!("Couldn't write output entry: {}", e);
        }
    }

    fn resize(&mut self, _winsize: &winsize::Winsize) {
        /* do something with winsize */
    }

    fn shutdown(&mut self) {
        /* prepare for shutdown */
        self.writer.flush().unwrap();
    }
}

// TODO: Unify this with structopts in main.
#[derive(Clone)]
pub struct Options {
    /// Title of the asciicast
    pub title: Option<String>,
    /// Limit recorded idle time to given number of seconds
    pub idle_time_limit: Option<f64>,
    /// Answer "yes" to all prompts (e.g. upload confirmation)
    pub force_yes: bool,
    /// Overwrite the file if it already exists
    pub overwrite: bool,
    /// Append to existing recording
    pub append: bool,
    /// Save only raw stdout output
    pub raw: bool,
    /// Filename/path to save the recording to
    pub file: Option<PathBuf>,
}

fn validate_output_path(settings: &RecordSettings) -> Result<(), Error> {
    match settings.file {
        Some(ref x) => {
            let exists = x.as_path().exists();
            // Create a new file if it doesn't exist or we were told to overwrite.
            if !exists || exists && settings.overwrite {
                return Ok(());
            }
            if exists && settings.append {
                // Append to existing file if we are told to do so.
                return Ok(());
            }

            Err(RecordFailure::FileExists {
                path: x.as_path().to_string_lossy().into_owned(),
            })?
        }
        None => Ok(()),
    }
}

pub fn go(settings: &RecordSettings, builder: &mut UploadBuilder) -> Result<RecordLocation, Error> {
    // First check to see if we should even start recording.
    validate_output_path(settings)?;

    let (cols, rows) = termion::terminal_size().context("Cannot get terminal size")?;

    // HACK: This is ugly, look away!
    // 1. We create a named temp file so we can get the path. Why? Because our uploader uses
    //    `reqwest` and it takes a `PathBuf`. It can take raw data but it needs to be `'static`
    //    and I can't get it to work with lifetimes, so we'll just make sure we always have a
    //    path.
    // 2. We get the path.
    // 3. We get a handle to the underlying file and send it to our writer. Why? Because the
    //    tempfile will be moved to the writer and then dropped before we can read from it.
    //    `tempfile` deletes files on `Drop`, so it is deleted before the rest of the program
    //    can process it.
    //
    // Sigh, I need to get better at Rust but this works.
    let tmp = NamedTempFile::new()?;
    let tmp_path = tmp.path().to_path_buf();
    let tmp_handle = tmp.reopen()?;

    let mut writer: LineWriter<Box<Write>> = LineWriter::new(Box::new(tmp_handle));

    // TODO: Now that we always write to a tempfile and we don't support streaming,
    // perhaps write the header at the end so we can fill out `duration`?
    let header = asciicast::Header {
        version: 2,
        width: u32::from(cols),
        height: u32::from(rows),
        timestamp: Some(Utc::now()),
        duration: None,
        idle_time_limit: settings.idle_time_limit,
        // TODO: Command support.
        command: None,
        title: settings.title.clone(),
    };
    let json_header = serde_json::to_string(&header).context("Cannot convert header to JSON")?;

    writeln!(writer, "{}", json_header).context("Cannot write header")?;

    let child = tty::Fork::from_ptmx()?;
    child.exec(env::var("SHELL").unwrap_or_else(|_| "sh".to_string()))?;

    // Write out the recording banner for interactive sessions.
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    writeln!(&mut stdout, "{}", "".to_string())?;
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
    let rec = format!(
        "{:\u{2B07}^1$}",
        "  \u{1F534}  [RECORDING]  ", cols as usize
    );
    writeln!(&mut stdout, "{}\n", rec)?;
    stdout.reset()?;
    stdout.flush()?;

    let shell = Shell {
        writer,
        clock: Instant::now(),
    };
    child.proxy(shell)?;
    child.wait()?;

    // Return where recorded asciicast can be found.
    Ok(match settings.file.clone() {
        Some(p) => {
            // Check again to see if we should write recording.
            validate_output_path(settings)?;
            // Move the temporary file into the user-specified path.
            tmp.persist(&p)?;
            RecordLocation::Local(p)
        }
        None => {
            // Upload the file to a remote service.
            // TODO: Prompt to upload like the python client does.
            let uploader = builder.build().map_err(err_msg)?;
            RecordLocation::Remote(uploader.upload_file(tmp_path)?)
        }
    })
}
