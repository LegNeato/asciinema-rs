use failure::Error;
use terminal::{Height, Width};
use std::collections::HashMap;
use std;

pub mod asciicast;
pub mod clock;
pub mod raw;

#[derive(Debug, Fail)]
pub enum SessionFailure {
    #[fail(display = "unable to write asciicast entry: {}", _0)]
    AsciicastEntryWrite(#[cause] std::io::Error),
    #[fail(display = "unable to write raw output: {}", _0)]
    RawOutputWrite(#[cause] std::io::Error),
}

pub trait Session {
    fn write_header(
        &mut self,
        height: &Height,
        width: &Width,
        idle_time_limit: Option<f64>,
        command: Option<String>,
        title: Option<String>,
        env: Option<HashMap<String, String>>,
    ) -> Result<(), Error>;
    fn write_output(&mut self, data: &[u8]) -> Result<(), Error>;
    fn write_input(&mut self, data: &[u8]) -> Result<(), Error>;
    fn end(&mut self) -> Result<(), Error>;
}
