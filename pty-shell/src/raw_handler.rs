use super::PtyHandler;
use mio::*;
use nix::sys::signal;
use std::io::Read;
use std::os::unix::io::AsRawFd;

use crate::tty;
use crate::winsize;

pub const INPUT: Token = Token(0);
pub const OUTPUT: Token = Token(1);

static mut SIGWINCH_COUNT: i32 = 0;
extern "C" fn handle_sigwinch(_: i32) {
    unsafe {
        SIGWINCH_COUNT += 1;
    }
}

pub struct RawHandler {
    pub input: unix::PipeReader,
    pub output: unix::PipeReader,
    pub pty: tty::Master,
    pub handler: Box<dyn PtyHandler>,
    pub resize_count: i32,
}

pub enum Message {
    Shutdown,
    Resize,
}

impl RawHandler {
    pub fn new(
        input: unix::PipeReader,
        output: unix::PipeReader,
        pty: tty::Master,
        handler: Box<dyn PtyHandler>,
    ) -> Self {
        RawHandler {
            input,
            output,
            pty,
            handler,
            resize_count: Self::sigwich_count(),
        }
    }

    pub fn register_sigwinch_handler() {
        let sig_action = signal::SigAction::new(
            signal::SigHandler::Handler(handle_sigwinch),
            signal::SaFlags::SA_RESTART,
            signal::SigSet::empty(),
        );

        unsafe {
            signal::sigaction(signal::SIGWINCH, &sig_action).unwrap();
        }
    }

    pub fn sigwich_count() -> i32 {
        unsafe { SIGWINCH_COUNT }
    }

    fn should_resize(&self) -> bool {
        let last = Self::sigwich_count();

        last > self.resize_count
    }
}

impl Handler for RawHandler {
    type Timeout = ();
    type Message = Message;

    fn ready(&mut self, event_loop: &mut EventLoop<RawHandler>, token: Token, events: EventSet) {
        match token {
            INPUT => {
                if events.is_readable() {
                    let mut buf = [0; 128];
                    let nread = self.input.read(&mut buf).unwrap();

                    (&mut *self.handler).input(&buf[..nread]);
                }
            }
            OUTPUT => {
                if events.is_readable() {
                    let mut buf = [0; 1024 * 10];
                    let nread = self.output.read(&mut buf).unwrap_or(0);

                    if nread == 0 {
                        event_loop.shutdown();
                    } else {
                        (&mut *self.handler).output(&buf[..nread]);
                    }
                }
            }
            _ => unimplemented!(),
        }
    }

    fn notify(&mut self, event_loop: &mut EventLoop<RawHandler>, message: Message) {
        match message {
            Message::Shutdown => {
                event_loop.shutdown();

                (&mut *self.handler).shutdown();
            }
            Message::Resize => {
                let winsize = winsize::from_fd(libc::STDIN_FILENO).unwrap();
                winsize::set(self.pty.as_raw_fd(), &winsize);

                (&mut *self.handler).resize(&winsize);

                self.resize_count = Self::sigwich_count();
            }
        }
    }

    fn tick(&mut self, event_loop: &mut EventLoop<RawHandler>) {
        if self.should_resize() {
            let _ = event_loop.channel().send(Message::Resize);
        }
    }
}

unsafe impl ::std::marker::Send for RawHandler {}
unsafe impl ::std::marker::Sync for RawHandler {}
