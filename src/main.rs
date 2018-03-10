extern crate asciicast;
extern crate chrono;
#[macro_use]
extern crate failure;
extern crate libc;
extern crate pty_shell;
extern crate termcolor;
extern crate url;
extern crate termion;

extern crate serde_json;
#[macro_use]
extern crate structopt;

use std::path::PathBuf;
use structopt::StructOpt;

mod commands;
use commands::record;

#[derive(StructOpt, Debug)]
#[structopt(name = "asciinema", author = "")]
/// Record and share your terminal sessions, the right way.
enum Opt {
    // TODO: Unify this with the command options.
    #[structopt(name = "rec")]
    /// Record terminal session
    RecordOptions {
        /// Title of the asciicast
        #[structopt(short = "t", long = "title")]
        title: Option<String>,
        /// Command to record, defaults to $SHELL
        #[structopt(short = "c", long = "command")]
        command: Option<String>,
        /// Limit recorded idle time to given number of seconds
        #[structopt(name = "IDLE_TIME_LIMIT", short = "i", long = "idle-time-limit")]
        idle_time_limit: Option<f64>,
        /// Answer "yes" to all prompts (e.g. upload confirmation)
        #[structopt(short = "y", long = "yes")]
        force_yes: bool,
        /// Overwrite the file if it already exists
        #[structopt(long = "overwrite")]
        overwrite: bool,
        /// Append to existing recording
        #[structopt(long = "append")]
        append: bool,
        /// Save only raw stdout output
        #[structopt(long = "raw")]
        raw: bool,
        /// Filename/path to save the recording to
        #[structopt(name = "FILE", parse(from_os_str))]
        file: Option<PathBuf>,
    },
}

fn main() {
    let result = match Opt::from_args() {
        // TODO: Is there a cleaner way to spread here?
        Opt::RecordOptions {
            title,
            command,
            idle_time_limit,
            force_yes,
            overwrite,
            append,
            raw,
            file,
        } => record::go(record::Options {
            title,
            command,
            idle_time_limit,
            force_yes,
            overwrite,
            append,
            raw,
            file,
        }),
    };

    std::process::exit(match result {
        Ok(location) => {
            match location {
                record::RecordLocation::Local(f) => {
                    println!("asciicast saved to: {}", f.to_string_lossy())
                }
                record::RecordLocation::Remote(url) => println!("{}", url),
            };
            0
        }
        Err(x) => {
            eprintln!("{}", x);
            1
        }
    })
}
