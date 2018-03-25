extern crate asciicast;
extern crate chrono;
extern crate serde_json;

use std::io::prelude::*;
use std::boxed::Box;
use failure::Error;
use terminal::{Height, Width};
use std::collections::HashMap;
use failure::ResultExt;
use std::str;
use super::clock::{get_elapsed_seconds, Clock};
use super::{Session, SessionFailure};

pub struct AsciicastSession<'a> {
    clock: Clock,
    writer: Box<Write + 'a>,
}

impl<'a> AsciicastSession<'a> {
    pub fn new(writer: Box<Write + 'a>) -> Self {
        AsciicastSession {
            clock: Clock::new(),
            writer: writer,
        }
    }
    fn get_elapsed_seconds(&self) -> f64 {
        get_elapsed_seconds(&self.clock.elapsed())
    }
    #[cfg(test)]
    pub(crate) fn set_clock(&mut self, c: Clock) {
        self.clock = c;
    }
}

impl<'a> Session for AsciicastSession<'a> {
    fn write_header(
        &mut self,
        height: &Height,
        width: &Width,
        idle_time_limit: Option<f64>,
        command: Option<String>,
        title: Option<String>,
        env: Option<HashMap<String, String>>,
    ) -> Result<(), Error> {
        // Generate asciicast header.
        let header = asciicast::Header {
            version: 2,
            width: width.0,
            height: height.0,
            timestamp: Some(self.clock.now()),
            duration: None,
            idle_time_limit,
            command,
            title,
            env,
        };
        // Serialize it to a JSON string.
        let json_header = serde_json::to_string(&header).context("Cannot convert header to JSON")?;

        // Write it out.
        writeln!(self.writer, "{}", json_header).context("Cannot write header")?;
        Ok(())
    }
    fn write_output(&mut self, data: &[u8]) -> Result<(), Error> {
        // Generate asciicast entry.
        let entry = asciicast::Entry {
            time: self.get_elapsed_seconds(),
            event_type: asciicast::EventType::Output,
            event_data: str::from_utf8(data)?.to_string(),
        };

        // Serialize it to a JSON string.
        let j = serde_json::to_string(&entry)?;

        // Write it out.
        writeln!(self.writer, "{}", j).map_err(SessionFailure::AsciicastEntryWrite)?;
        Ok(())
    }
    fn write_input(&mut self, _data: &[u8]) -> Result<(), Error> {
        println!("INPUT");
        Ok(())
    }
    fn end(&mut self) -> Result<(), Error> {
        self.writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use chrono::{DateTime, Utc};

    fn write_asciicast_header(
        now: DateTime<Utc>,
        height: &Height,
        width: &Width,
        idle_time_limit: Option<f64>,
        command: Option<String>,
        title: Option<String>,
        env: Option<HashMap<String, String>>,
    ) -> String {
        // Tests write to memory.
        let mut v = Vec::new();

        // New scope so that our Vec can be read at the end.
        {
            let mut session = AsciicastSession::new(Box::new(&mut v));
            let mut clock = Clock::new();
            clock.set_manual_now(now);
            session.set_clock(clock);
            // Write header.
            session
                .write_header(height, width, idle_time_limit, command, title, env)
                .unwrap();
        }
        String::from_utf8(v).unwrap()
    }

    fn write_asciicast_output(data: &str, duration: Duration) -> String {
        // Tests write to memory.
        let mut v = Vec::new();

        // New scope so that our Vec can be read at the end.
        {
            let mut session = AsciicastSession::new(Box::new(&mut v));
            let mut clock = Clock::new();
            clock.set_manual_duration(duration);
            session.set_clock(clock);

            // Write data.
            session.write_output(data.as_bytes()).unwrap();
        }
        String::from_utf8(v).unwrap()
    }

    #[test]
    fn writes_header() {
        let now = Utc::now();
        let result = write_asciicast_header(
            now.clone(),
            &Height(42),
            &Width(161),
            None,
            None,
            Some("test title".to_string()),
            None,
        );
        let expected = format!(
            "{{\
             \"version\":2,\
             \"width\":161,\
             \"height\":42,\
             \"timestamp\":{},\
             \"title\":\"test title\"\
             }}\n",
            now.timestamp()
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn writes_output() {
        let result = write_asciicast_output("Hello world", Duration::new(5, 0));
        assert_eq!(result, "[5.0,\"o\",\"Hello world\"]\n".to_string());
    }
}
