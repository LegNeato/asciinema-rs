use failure::Error;
use std::collections::HashMap;
use terminal::{Height, Width};
extern crate asciicast;
extern crate chrono;
extern crate serde_json;
use self::clock::{get_elapsed_seconds, Clock};
use output_formats::Msg;
use std::boxed::Box;
use std::str;
use std::sync::mpsc::Sender;

pub mod clock;

pub struct Session {
    clock: Clock,
    outputs: Vec<Sender<Msg>>,
}

impl Session {
    pub fn new(outputs: Vec<Sender<Msg>>) -> Self {
        Session {
            clock: Clock::new(),
            outputs,
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

impl Session {
    pub fn write_header(
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
        for channel in &self.outputs {
            channel.send(Msg::Header(Box::new(header.clone())))?;
        }
        Ok(())
    }
    pub fn write_output(&mut self, data: &[u8]) -> Result<(), Error> {
        // Generate asciicast entry.
        let entry = asciicast::Entry {
            time: self.get_elapsed_seconds(),
            event_type: asciicast::EventType::Output,
            event_data: str::from_utf8(data)?.to_string(),
        };

        // Write it out.
        for channel in &self.outputs {
            channel.send(Msg::Output(Box::new(entry.clone())))?;
        }
        Ok(())
    }
    #[allow(unused)]
    pub fn write_input(&mut self, _data: &[u8]) -> Result<(), Error> {
        println!("INPUT");
        Ok(())
    }
    pub fn end(&mut self) -> Result<(), Error> {
        for channel in &self.outputs {
            channel.send(Msg::Finish)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asciicast::{Entry, EventType, Header};
    use chrono::{DateTime, Utc};
    use std::boxed::Box;
    use std::sync::mpsc::channel;
    use std::time::Duration;
    use terminal::{Height, Width};

    fn make_mock_session(
        tx: Sender<Msg>,
        now: Option<DateTime<Utc>>,
        duration: Option<Duration>,
    ) -> Session {
        let mut session = Session::new(vec![tx]);
        // Make clock deterministic
        let mut clock = Clock::new();
        if let Some(n) = now {
            clock.set_now_override(n.clone());
        }
        if let Some(d) = duration {
            clock.set_duration_override(d);
        }
        session.set_clock(clock);
        session
    }

    #[test]
    fn sends_message_for_header() {
        let now = Utc::now();
        let (tx, rx) = channel();
        let mut session = make_mock_session(tx, Some(now), None);

        session
            .write_header(
                &Height(42),
                &Width(161),
                None,
                None,
                Some("test title".to_string()),
                None,
            )
            .unwrap();

        let result = rx.try_recv();
        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(
            message,
            Msg::Header(Box::new(Header {
                version: 2,
                width: 161,
                height: 42,
                command: None,
                duration: None,
                env: None,
                idle_time_limit: None,
                title: Some("test title".to_string()),
                timestamp: Some(now),
            }))
        );
    }

    #[test]
    fn sends_message_for_output() {
        let duration = Duration::new(5, 0);
        let (tx, rx) = channel();
        let mut session = make_mock_session(tx, None, Some(duration));

        session
            .write_output("hello world".to_string().as_bytes())
            .unwrap();

        let result = rx.try_recv();
        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(
            message,
            Msg::Output(Box::new(Entry {
                event_type: EventType::Output,
                event_data: "hello world".to_string(),
                time: 5.0,
            }))
        );
    }
}
