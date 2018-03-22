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
use std::collections::HashMap;
use std::str;
use uploader::UploadBuilder;
use std::io::LineWriter;
use pty_shell::*;
use std::time::{Duration, Instant};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::path::PathBuf;
use std::result::Result;
use failure::{err_msg, Error};
use failure::ResultExt;
use url::Url;
use termion;
use std;
use tempfile::NamedTempFile;
use settings::RecordSettings;

#[derive(Debug, Fail)]
enum RecordFailure {
    #[fail(display = "unable to write to file: {}: file exists", path)]
    FileExists { path: String },
    #[fail(display = "unable to write asciicast entry: {}", _0)]
    AsciicastEntryWrite(#[cause] std::io::Error),
    #[fail(display = "unable to write raw output: {}", _0)]
    RawOutputWrite(#[cause] std::io::Error),
}

fn get_elapsed_seconds(duration: &Duration) -> f64 {
    duration.as_secs() as f64 + (0.000_000_001 * f64::from(duration.subsec_nanos()))
}

fn capture_environment_vars(keys: Vec<&str>) -> HashMap<String, String> {
    let mut h = HashMap::new();
    for key in keys {
        if let Ok(value) = env::var(&key) {
            h.insert(key.to_string(), value);
        }
    }
    h
}

fn get_environment_for_child<I>(parent_env: I) -> HashMap<String, String>
where
    I: Iterator<Item = (String, String)>,
{
    // Duplicate the parent environment.
    let mut child_env = HashMap::new();
    for (key, value) in parent_env {
        child_env.insert(key, value);
    }
    // Add `ASCIINEMA_REC` env variable.
    child_env.insert("ASCIINEMA_REC".to_string(), "1".to_string());
    child_env
}

fn write_asciicast_event<W: ?Sized>(
    writer: &mut LineWriter<Box<W>>,
    event_type: asciicast::EventType,
    since_start: Duration,
    data: &[u8],
) -> Result<(), Error>
where
    W: Write,
{
    // Generate asciicast entry.
    let entry = asciicast::Entry {
        time: get_elapsed_seconds(&since_start),
        event_type,
        event_data: str::from_utf8(data)?.to_string(),
    };

    // Serialize it to a JSON string.
    let j = serde_json::to_string(&entry)?;

    // Write it out.
    writeln!(writer, "{}", j).map_err(RecordFailure::AsciicastEntryWrite)?;
    Ok(())
}

fn write_raw_output<W: ?Sized>(writer: &mut LineWriter<Box<W>>, data: &[u8]) -> Result<(), Error>
where
    W: Write,
{
    let raw_out = str::from_utf8(data)?;
    write!(writer, "{}", raw_out).map_err(RecordFailure::RawOutputWrite)?;
    Ok(())
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
    raw: bool,
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
        if self.raw {
            write_raw_output(self.writer.by_ref(), output).unwrap();
        } else {
            write_asciicast_event(
                self.writer.by_ref(),
                asciicast::EventType::Output,
                self.clock.elapsed(),
                output,
            ).unwrap();
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

    if !settings.raw {
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
            env: Some(capture_environment_vars(vec!["SHELL", "TERM"])),
        };
        let json_header = serde_json::to_string(&header).context("Cannot convert header to JSON")?;
        writeln!(writer, "{}", json_header).context("Cannot write header")?;
    }

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

    let child = tty::Fork::from_ptmx()?;
    let shell = Shell {
        writer,
        clock: Instant::now(),
        raw: settings.raw,
    };
    child.proxy(shell)?;

    let child_env = get_environment_for_child(env::vars());

    child.exec_with_env(
        env::var("SHELL").unwrap_or_else(|_| "sh".to_string()),
        Some(child_env),
    )?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use asciicast;
    use std::io::Cursor;
    use std::io::LineWriter;
    use std::time::Duration;
    use std::any::Any;
    use settings::RecordSettings;
    use std::path::PathBuf;

    enum FileBehavior {
        NotSet,
        Append,
        Overwrite,
    }

    fn get_mock_settings(file: Option<PathBuf>, behavior: FileBehavior) -> RecordSettings {
        let mut append = false;
        let mut overwrite = false;
        match behavior {
            FileBehavior::Append => append = true,
            FileBehavior::Overwrite => overwrite = true,
            FileBehavior::NotSet => (),
        };
        RecordSettings {
            append,
            overwrite,
            file,
            force_yes: false,
            idle_time_limit: None,
            raw: false,
            title: None,
        }
    }

    fn write_mock_asciicast_event(
        event_type: asciicast::EventType,
        duration: Duration,
        data: String,
    ) -> String {
        // Tests write to memory.
        let c = Cursor::new(vec![0; 15]);
        let mut writer = LineWriter::new(Box::new(c));

        // Serialize and write the event.
        write_asciicast_event(&mut writer, event_type, duration, data.as_bytes()).unwrap();

        // First we get our Box from the LineWriter.
        let box_from_writer: Box<Any> = writer.into_inner().unwrap();
        // Then we get our Cursor from the Box.
        let c = box_from_writer.downcast::<Cursor<Vec<u8>>>().unwrap();
        // Then we get the Vec from the Cursor.
        let buff = c.into_inner();

        // The Vec contains what was written...return it as a String.
        String::from_utf8(buff).unwrap()
    }

    fn write_mock_raw_output(data: String) -> String {
        let c = Cursor::new(vec![0; 11]);
        let mut writer = LineWriter::new(Box::new(c));

        // Write the raw output.
        write_raw_output(&mut writer, data.as_bytes()).unwrap();

        let box_from_writer: Box<Any> = writer.into_inner().unwrap();
        let c = box_from_writer.downcast::<Cursor<Vec<u8>>>().unwrap();
        let buff = c.into_inner();

        String::from_utf8(buff).unwrap()
    }

    #[test]
    fn test_elapsed_whole_seconds() {
        let d = Duration::new(5, 0);
        let result = get_elapsed_seconds(&d);
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_elapsed_fractional_seconds() {
        let d = Duration::new(42, 123);
        let result = get_elapsed_seconds(&d);
        assert_eq!(result, 42.000000123);
    }

    #[test]
    fn test_writing_asciicast_output_event() {
        let result = write_mock_asciicast_event(
            asciicast::EventType::Output,
            Duration::new(123, 456),
            "Hello world".to_string(),
        );
        assert_eq!(result, "[123.000000456,\"o\",\"Hello world\"]\n");
    }

    #[test]
    fn test_writing_raw_output() {
        let result = write_mock_raw_output("Hello world".to_string());
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_unset_output_path() {
        let result = validate_output_path(&get_mock_settings(None, FileBehavior::NotSet));
        assert!(result.is_ok());
    }

    #[test]
    fn test_nonexistent_output_path() {
        let result = validate_output_path(&get_mock_settings(
            Some(PathBuf::from("/does_not_exist.txt")),
            FileBehavior::NotSet,
        ));
        assert!(result.is_ok());
    }

    #[test]
    fn test_nonexistent_output_path_with_append() {
        let result = validate_output_path(&get_mock_settings(
            Some(PathBuf::from("/does_not_exist.txt")),
            FileBehavior::Append,
        ));
        assert!(result.is_ok());
    }

    #[test]
    fn test_existent_output_path() {
        let result = validate_output_path(&get_mock_settings(
            // Current directory always exists.
            Some(PathBuf::from(".")),
            FileBehavior::NotSet,
        ));
        // TODO: Figure out a better way to check this.
        assert_eq!(
            format!("{}", result.unwrap_err()),
            format!(
                "{}",
                RecordFailure::FileExists {
                    path: ".".to_string(),
                }
            ),
        );
    }

    #[test]
    fn test_existent_output_path_with_overwrite() {
        let result = validate_output_path(&get_mock_settings(
            // Current directory always exists.
            Some(PathBuf::from(".")),
            FileBehavior::Overwrite,
        ));
        assert!(result.is_ok());
    }

    #[test]
    fn test_existent_output_path_with_append() {
        let result = validate_output_path(&get_mock_settings(
            // Current directory always exists.
            Some(PathBuf::from(".")),
            FileBehavior::Append,
        ));
        assert!(result.is_ok());
    }

    #[test]
    fn test_capturing_env_for_header() {
        env::set_var("THIS_IS_A_TEST_1", "1");
        env::set_var("THIS_IS_A_TEST_2", "2");
        let result = capture_environment_vars(vec![
            "THIS_IS_A_TEST_1",
            "THIS_IS_A_TEST_2",
            "THIS_IS_A_TEST_3",
        ]);
        assert_eq!(result.get("THIS_IS_A_TEST_1"), Some(&"1".to_string()));
        assert_eq!(result.get("THIS_IS_A_TEST_2"), Some(&"2".to_string()));
        assert_eq!(result.get("THIS_IS_A_TEST_3"), None);
    }

    #[test]
    fn test_env_copies_parent() {
        let mut e = HashMap::new();
        e.insert("FOO".to_string(), "test".to_string());
        e.insert("BAR".to_string(), "1".to_string());
        let result = get_environment_for_child(e.iter().map(|(a, b)| (a.clone(), b.clone())));
        assert_eq!(result.get("FOO"), Some(&"test".to_string()));
        assert_eq!(result.get("BAR"), Some(&"1".to_string()));
    }

    #[test]
    fn test_env_sets_asciinema_rec() {
        let e: HashMap<String, String> = HashMap::new();
        let result = get_environment_for_child(e.iter().map(|(a, b)| (a.clone(), b.clone())));
        assert_eq!(result.get("ASCIINEMA_REC"), Some(&"1".to_string()));
    }
}
