extern crate asciicast;
extern crate chrono;
extern crate config;
#[macro_use]
extern crate failure;
extern crate libc;
extern crate pty_shell;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate structopt;
extern crate termcolor;
extern crate termion;
extern crate url;
extern crate url_serde;
extern crate uuid;

mod commands;
mod settings;

use settings::{Action, Settings};
use commands::record::RecordLocation;
use failure::Error;
use url::Url;

enum CommandResult {
    Authenticate(Result<Url, Error>),
    Record(Result<RecordLocation, Error>),
}

fn main() {
    let settings = Settings::new().unwrap();

    let result = match settings.action {
        Action::Authenticate => CommandResult::Authenticate(commands::authenticate::go(
            settings.authenticate.unwrap(),
            settings.api_url,
        )),
        Action::Record => CommandResult::Record(commands::record::go(
            settings.record.unwrap(),
            settings.api_url,
        )),
    };

    std::process::exit(match result {
        CommandResult::Record(x) => match x {
            Ok(location) => {
                let location_output = match location {
                    RecordLocation::Local(f) => {
                        format!("asciicast saved to: {}", f.to_string_lossy())
                    }
                    RecordLocation::Remote(url) => format!("{}", url),
                };
                handle_output(location_output)
            }
            Err(x) => handle_error(x),
        },
        CommandResult::Authenticate(x) => match x {
            Ok(url) => handle_output(format!(
                "Open the following URL in a web browser to \
                 link your install ID with your asciinema.org user account:\
                 \n\n{}\n\n\
                 This will associate all recordings uploaded from this machine \
                 (past and future ones) to your account, and allow you to manage \
                 them (change title/theme, delete) at asciinema.org.",
                url
            )),
            Err(x) => handle_error(x),
        },
    })
}

fn handle_output(s: String) -> i32 {
    println!("{}", s);
    // If we don't do this, the prompt when we exit is too far right. ¯\_(ツ)_/¯
    print!("{}", termion::cursor::Left(s.chars().count() as u16));
    0
}

fn handle_error(e: Error) -> i32 {
    eprintln!("{}", e);
    1
}
