use ::asciicast as asciicast_format;
use failure::Error;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

#[cfg(test)]
#[macro_use]
pub mod test_helpers;
pub mod asciicast;
pub mod raw;

#[derive(Debug, PartialEq, Clone)]
pub enum Msg {
    Header(Box<asciicast_format::Header>),
    Input(Box<asciicast_format::Entry>),
    Output(Box<asciicast_format::Entry>),
    Finish,
    #[cfg(test)]
    MockError,
}

pub enum LoopAction {
    Stop,
    Continue,
}

pub trait Output {
    fn channel(&self) -> Sender<Msg>;
    fn handle_message(&self, message: Msg) -> Result<LoopAction, Error>;
    fn rx(&self) -> &Receiver<Msg>;
    fn spawn(self) -> thread::JoinHandle<Result<(), Error>>
    where
        Self: Sized + Send + 'static,
    {
        thread::spawn(move || {
            let rx = self.rx();
            for message in rx.iter() {
                match self.handle_message(message)? {
                    LoopAction::Continue => (),
                    LoopAction::Stop => break,
                }
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use failure::Error;
    use std::sync::mpsc::channel;

    #[derive(Debug, Fail)]
    enum MockFailure {
        #[fail(display = "mock fail")]
        Fail,
    }

    struct Mock {
        tx: Sender<Msg>,
        rx: Receiver<Msg>,
    }

    impl Output for Mock {
        fn channel(&self) -> Sender<Msg> {
            self.tx.clone()
        }
        fn handle_message(&self, message: Msg) -> Result<LoopAction, Error> {
            match message {
                Msg::Finish => Ok(LoopAction::Stop),
                Msg::MockError => Err(MockFailure::Fail {})?,
                _ => Ok(LoopAction::Continue),
            }
        }
        fn rx(&self) -> &Receiver<Msg> {
            &self.rx
        }
    }

    #[test]
    fn thread_stops_on_finish_message() {
        let (tx, rx) = channel();
        let output = Mock { tx: tx.clone(), rx };
        let thread_handle = output.spawn();
        tx.send(Msg::Finish).expect("send message");
        // This should join / not wait forever.
        thread_handle.join().expect("thread join").unwrap();
    }

    #[test]
    fn thread_stops_on_error() {
        let (tx, rx) = channel();
        let output = Mock { tx: tx.clone(), rx };
        let thread_handle = output.spawn();
        tx.send(Msg::MockError).expect("send message");
        // This should join / not wait forever.
        let result = thread_handle.join().expect("thread join");
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            format!("{}", MockFailure::Fail {})
        );
    }
}
