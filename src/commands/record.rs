extern crate asciicast;
extern crate chrono;
extern crate libc;
extern crate pty_shell;
extern crate termcolor;
extern crate url;

extern crate serde_json;

use failure::ResultExt;
use failure::{err_msg, Error};
use pty_shell::*;
use session::Session;
use session::asciicast::AsciicastSession;
use session::raw::RawSession;
use settings::RecordSettings;
use std::collections::HashMap;
use std::env;
use std::fs::OpenOptions;
use std::io::LineWriter;
use std::io::prelude::*;
use std::path::PathBuf;
use std::result::Result;
use std::str;
use tempfile::NamedTempFile;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use terminal::{Height, Width};
use termion;
use uploader::UploadBuilder;
use url::Url;

#[derive(Debug, Fail)]
enum RecordFailure {
    #[fail(display = "unable to write to file: {}: file exists", path)]
    FileExists { path: String },
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

pub enum RecordLocation {
    Local(PathBuf),
    Remote(Url),
}

struct Shell {
    session: Box<Session>,
}

impl PtyHandler for Shell {
    fn input(&mut self, _input: &[u8]) {
        /* do something with input */
        //println!("In: {:?}", input);
    }

    fn output(&mut self, output: &[u8]) {
        self.session
            .write_output(output)
            .expect("unable to write output");
    }

    fn resize(&mut self, _winsize: &winsize::Winsize) {
        /* do something with winsize */
    }

    fn shutdown(&mut self) {
        /* prepare for shutdown */
        self.session.end().expect("unable to end session");
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
    let handle = match settings.file.clone() {
        Some(p) => OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(settings.overwrite)
            .open(&p)?,
        None => tmp.reopen()?,
    };

    let mut session: Box<Session> = if settings.raw {
        Box::new(RawSession::new(Box::new(LineWriter::new(handle))))
    } else {
        Box::new(AsciicastSession::new(Box::new(LineWriter::new(handle))))
    };

    session.write_header(
        &Height(u32::from(rows)),
        &Width(u32::from(cols)),
        settings.idle_time_limit,
        None, // TODO: Command.
        settings.title.clone(),
        Some(capture_environment_vars(vec!["SHELL", "TERM"])),
    )?;

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
    restore_termios();
    let shell = Shell { session };
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
            // Written to the user-specified path.
            RecordLocation::Local(p)
        }
        None => {
            // Upload the temp file to a remote service.
            // TODO: Prompt to upload like the python client does.
            let uploader = builder.build().map_err(err_msg)?;
            RecordLocation::Remote(uploader.upload_file(tmp_path)?)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
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
