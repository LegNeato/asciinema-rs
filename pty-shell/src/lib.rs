#![deny(missing_debug_implementations,
        trivial_casts, trivial_numeric_casts,
        unstable_features,
        unused_import_braces, unused_qualifications)]

extern crate libc;
extern crate nix;
extern crate pty;
extern crate termios;
extern crate mio;

pub use pty::prelude as tty;

pub use self::error::*;
use self::raw_handler::*;
pub use self::terminal::*;
use self::winsize::Winsize;
use std::{fmt, io, result, thread};
use std::io::{Read, Write};

pub mod error;
pub mod terminal;
pub mod winsize;

mod raw_handler;
mod command;

pub type Result<T> = result::Result<T, Error>;

pub trait PtyHandler {
    fn input(&mut self, _data: &[u8]) {}
    fn output(&mut self, _data: &[u8]) {}
    fn resize(&mut self, _winsize: &Winsize) {}
    fn shutdown(&mut self) {}
}

pub trait PtyShell {
    fn exec<S: AsRef<str>>(&self, shell: S) -> Result<()>;
    fn proxy<H: PtyHandler + 'static>(&self, handler: H) -> Result<()>;
}

impl PtyShell for tty::Fork {
    fn exec<S: AsRef<str>>(&self, shell: S) -> Result<()> {
        if self.is_child().is_ok() {
            command::exec(shell);
        }

        Ok(())
    }

    fn proxy<H: PtyHandler + 'static>(&self, handler: H) -> Result<()> {
        if let Some(master) = self.is_parent().ok() {
            try!(setup_terminal(master));
            try!(do_proxy(master, handler));
        }
        Ok(())
    }
}

fn do_proxy<H: PtyHandler + 'static>(pty: tty::Master, handler: H) -> Result<()> {
    let mut event_loop = try!(mio::EventLoop::new());

    let mut writer = pty.clone();
    let (input_reader, mut input_writer) = try!(mio::unix::pipe());

    thread::spawn(move || {
        handle_input(&mut writer, &mut input_writer).unwrap_or_else(|e| {
            println!("{:?}", e);
        });
    });

    let mut reader = pty.clone();
    let (output_reader, mut output_writer) = try!(mio::unix::pipe());
    let message_sender = event_loop.channel();

    thread::spawn(move || {
        handle_output(&mut reader, &mut output_writer).unwrap_or_else(|e| {
            println!("{:?}", e);
        });

        message_sender.send(Message::Shutdown).unwrap();
    });

    try!(event_loop.register(&input_reader,
                             INPUT,
                             mio::EventSet::readable(),
                             mio::PollOpt::level()));
    try!(event_loop.register(&output_reader,
                             OUTPUT,
                             mio::EventSet::readable(),
                             mio::PollOpt::level()));
    RawHandler::register_sigwinch_handler();

    let mut raw_handler =
        RawHandler::new(input_reader, output_reader, pty.clone(), Box::new(handler));

    thread::spawn(move || {
        event_loop.run(&mut raw_handler).unwrap_or_else(|e| {
            println!("{:?}", e);
        });
    });

    Ok(())
}

fn handle_input(writer: &mut tty::Master,
                handler_writer: &mut mio::unix::PipeWriter)
                -> Result<()> {
    let mut input = io::stdin();
    let mut buf = [0; 128];

    loop {
        let nread = try!(input.read(&mut buf));

        try!(writer.write(&buf[..nread]));
        try!(handler_writer.write(&buf[..nread]));
    }
}

fn handle_output(reader: &mut tty::Master,
                 handler_writer: &mut mio::unix::PipeWriter)
                 -> Result<()> {
    let mut output = io::stdout();
    let mut buf = [0; 1024 * 10];

    loop {
        let nread = try!(reader.read(&mut buf));

        if nread <= 0 {
            break;
        } else {
            try!(output.write(&buf[..nread]));
            let _ = output.flush();

            try!(handler_writer.write(&buf[..nread]));
        }
    }

    Ok(())
}

pub struct PtyCallbackData {
    input_handler: Box<FnMut(&[u8])>,
    output_handler: Box<FnMut(&[u8])>,
    resize_handler: Box<FnMut(&Winsize)>,
    shutdown_handler: Box<FnMut()>,
}

impl fmt::Debug for PtyCallbackData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(PtyCallbackData)")
    }
}

impl PtyHandler for PtyCallbackData {
    fn input(&mut self, data: &[u8]) {
        (&mut *self.input_handler)(data);
    }

    fn output(&mut self, data: &[u8]) {
        (&mut *self.output_handler)(data);
    }

    fn resize(&mut self, size: &Winsize) {
        (&mut *self.resize_handler)(size);
    }

    fn shutdown(&mut self) {
        (&mut *self.shutdown_handler)();
    }
}

#[derive(Debug)]
pub struct PtyCallbackBuilder(PtyCallbackData);

impl PtyCallbackBuilder {
    pub fn new() -> Self {
        let data = PtyCallbackData {
            input_handler: Box::new(|_| {}),
            output_handler: Box::new(|_| {}),
            resize_handler: Box::new(|_| {}),
            shutdown_handler: Box::new(|| {}),
        };

        PtyCallbackBuilder(data)
    }

    pub fn input<F>(mut self, handler: F) -> Self
        where F: FnMut(&[u8]) + 'static
    {
        self.0.input_handler = Box::new(handler);

        self
    }

    pub fn output<F>(mut self, handler: F) -> Self
        where F: FnMut(&[u8]) + 'static
    {
        self.0.output_handler = Box::new(handler);

        self
    }

    pub fn resize<F>(mut self, handler: F) -> Self
        where F: FnMut(&Winsize) + 'static
    {
        self.0.resize_handler = Box::new(handler);

        self
    }

    pub fn shutdown<F>(mut self, handler: F) -> Self
        where F: FnMut() + 'static
    {
        self.0.shutdown_handler = Box::new(handler);

        self
    }

    pub fn build(self) -> PtyCallbackData {
        self.0
    }
}

pub type PtyCallback = PtyCallbackBuilder;
