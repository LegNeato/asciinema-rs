extern crate asciicast;
extern crate chrono;
extern crate serde_json;

use super::{Session, SessionFailure};
use failure::Error;
use std::boxed::Box;
use std::collections::HashMap;
use std::io::prelude::*;
use std::str;
use terminal::{Height, Width};

pub struct RawSession<'a> {
    writer: Box<Write + 'a>,
}

impl<'a> RawSession<'a> {
    pub fn new(writer: Box<Write + 'a>) -> Self {
        RawSession { writer }
    }
}

impl<'a> Session for RawSession<'a> {
    fn write_header(
        &mut self,
        _height: &Height,
        _width: &Width,
        _idle_time_limit: Option<f64>,
        _command: Option<String>,
        _title: Option<String>,
        _env: Option<HashMap<String, String>>,
    ) -> Result<(), Error> {
        // No-op. Raw doesn't write a header.
        Ok(())
    }
    fn write_output(&mut self, data: &[u8]) -> Result<(), Error> {
        let raw_out = str::from_utf8(data)?;
        write!(self.writer, "{}", raw_out).map_err(SessionFailure::RawOutputWrite)?;
        Ok(())
    }
    fn write_input(&mut self, _data: &[u8]) -> Result<(), Error> {
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
    use terminal::{Height, Width};

    #[test]
    fn writes_header() {
        // Tests write to memory.
        let mut v = Vec::new();

        // New scope so that our Vec can be read at the end.
        {
            let mut session = RawSession::new(Box::new(&mut v));
            // Write header.
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
        }
        let result = String::from_utf8(v).unwrap();
        assert_eq!(result, "".to_string());
    }

    #[test]
    fn writes_output() {
        // Tests write to memory.
        let mut v = Vec::new();

        // New scope so that our Vec can be read at the end.
        {
            let mut session = RawSession::new(Box::new(&mut v));
            // Write header.
            session.write_output("Hello world".as_bytes()).unwrap();
        }
        let result = String::from_utf8(v).unwrap();
        assert_eq!(result, "Hello world".to_string());
    }
}
