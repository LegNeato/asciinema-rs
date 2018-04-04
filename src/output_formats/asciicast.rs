extern crate asciicast;
extern crate serde_json;

use super::{LoopAction, Msg, Output};
use failure::Error;
use std::fs::File;
use std::io::Write;
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct AsciicastOutput {
    rx: Receiver<Msg>,
    tx: Sender<Msg>,
    output: File,
}

impl AsciicastOutput {
    pub fn new(output: File) -> Self {
        let (tx, rx) = channel();
        AsciicastOutput { rx, tx, output }
    }
}

impl Output for AsciicastOutput {
    fn channel(&self) -> Sender<Msg> {
        self.tx.clone()
    }
    fn rx(&self) -> &Receiver<Msg> {
        &self.rx
    }
    fn handle_message(&self, message: Msg) -> Result<LoopAction, Error> {
        match message {
            Msg::Finish => Ok(LoopAction::Stop),
            Msg::Header(d) => {
                let mut file_copy = self.output.try_clone()?;
                let j = serde_json::to_string(&d)? + "\n";
                file_copy.write_all(j.as_bytes())?;
                Ok(LoopAction::Continue)
            }
            Msg::Input(d) | Msg::Output(d) => {
                let mut file_copy = self.output.try_clone()?;
                let j = serde_json::to_string(&d)? + "\n";
                file_copy.write_all(j.as_bytes())?;
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
    fn test_writes_header() {
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

        let line = first_line_for_message!(AsciicastOutput, message);
        assert!(line.is_some());
        assert_eq!(
            line.expect("a line").unwrap(),
            "{\
             \"version\":2,\
             \"width\":161,\
             \"height\":42\
             }",
        );
    }

    #[test]
    fn test_writes_output_event() {
        let entry = Entry {
            event_type: EventType::Output,
            event_data: "Hello world".to_string(),
            time: 5.0,
        };
        let line = first_line_for_message!(AsciicastOutput, Msg::Output(Box::new(entry.clone())));
        assert!(line.is_some());
        assert_eq!(
            line.expect("a line").unwrap(),
            "[5.0,\"o\",\"Hello world\"]".to_string()
        );
    }
}
