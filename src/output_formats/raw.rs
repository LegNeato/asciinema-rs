extern crate asciicast;
extern crate serde_json;

use super::{LoopAction, Msg, Output};
use failure::Error;
use std::fs::File;
use std::io::Write;
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct RawOutput {
    rx: Receiver<Msg>,
    tx: Sender<Msg>,
    output: File,
}

impl RawOutput {
    pub fn new(output: File) -> Self {
        let (tx, rx) = channel();
        RawOutput { rx, tx, output }
    }
}

impl Output for RawOutput {
    fn channel(&self) -> Sender<Msg> {
        self.tx.clone()
    }
    fn rx(&self) -> &Receiver<Msg> {
        &self.rx
    }
    fn handle_message(&self, message: Msg) -> Result<LoopAction, Error> {
        match message {
            Msg::Finish => Ok(LoopAction::Stop),
            Msg::Header(_) => {
                // Raw doesn't write header.
                Ok(LoopAction::Continue)
            }
            Msg::Input(entry) | Msg::Output(entry) => {
                let mut file_copy = self.output.try_clone()?;
                let data = (*entry).event_data;
                file_copy.write_all(data.as_bytes())?;
                Ok(LoopAction::Continue)
            }
            #[cfg(test)]
            Msg::MockError => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asciicast::{Entry, EventType, Header};
    use std::boxed::Box;

    #[test]
    fn test_does_not_write_header() {
        let message = Msg::Header(Box::new(Header {
            version: 2,
            width: 161,
            height: 42,
            command: None,
            duration: None,
            env: None,
            idle_time_limit: None,
            title: None,
            timestamp: None,
        }));

        let line = first_line_for_message!(RawOutput, message);
        assert!(line.is_none());
    }

    #[test]
    fn test_writes_output_event() {
        let entry = Entry {
            event_type: EventType::Output,
            event_data: "hello world".to_string(),
            time: 5.0,
        };
        let line = first_line_for_message!(RawOutput, Msg::Output(Box::new(entry.clone())));
        assert!(line.is_some());
        assert_eq!(line.expect("a line").unwrap(), entry.event_data);
    }
}
