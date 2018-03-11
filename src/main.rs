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

mod commands;
use commands::record;

mod settings;
use settings::{Action, Settings};
use record::RecordLocation;

fn main() {
    let settings = Settings::new().unwrap();

    let result = match settings.action {
        Action::Record => record::go(settings.record, settings.api_url),
        // TODO: other actions.
    };

    std::process::exit(match result {
        Ok(location) => {
            let location_output = match location {
                RecordLocation::Local(f) => {
                    format!("asciicast saved to: {}", f.to_string_lossy())
                }
                RecordLocation::Remote(url) => format!("{}", url),
            };
            println!("{}", location_output);
            // If we don't do this, the prompt when we exit is too far right.
            print!(
                "{}",
                termion::cursor::Left(location_output.chars().count() as u16)
            );
            0
        }
        Err(x) => {
            eprintln!("{}", x);
            1
        }
    })
}
