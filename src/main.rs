mod api;
mod clock;
mod commands;
mod output_formats;
mod session;
mod settings;
mod terminal;
mod uploader;

use api::Api;
use commands::record::RecordLocation;
use failure::Error;
use settings::install::InstallInfo;
use settings::{Action, Settings};
use uploader::UploadBuilder;
use url::Url;

enum CommandResult {
    Authenticate(Result<Url, Error>),
    Concatenate(Result<(), Error>),
    Play(Result<(), Error>),
    Record(Result<RecordLocation, Error>),
    Upload(Result<Url, Error>),
}

fn main() {
    // Set proper env vars when statically compiled for musl on Linux.
    openssl_probe::init_ssl_cert_env_vars();

    let settings = Settings::new().unwrap();
    let api = Api::new(&settings.api_url).unwrap();
    // Load install id from a file or generate a new one.
    // Note: the reference python version doesn't fail when
    // there is no existing install id, so we don't either.
    let install_info = InstallInfo::new().unwrap();

    let result = match settings.action {
        Action::Authenticate => CommandResult::Authenticate(commands::authenticate::go(
            &settings.authenticate.unwrap(),
            api,
        )),
        Action::Concatenate => {
            CommandResult::Concatenate(commands::concatenate::go(&settings.concatenate.unwrap()))
        }
        Action::Play => CommandResult::Play(commands::play::go(&settings.play.unwrap())),
        Action::Record => CommandResult::Record(commands::record::go(
            &settings.record.unwrap(),
            UploadBuilder::default()
                .api(api)
                .install_id(install_info.id),
        )),
        Action::Upload => CommandResult::Upload(commands::upload::go(
            &settings.upload.unwrap(),
            UploadBuilder::default()
                .api(api)
                .install_id(install_info.id),
        )),
    };

    std::process::exit(match result {
        CommandResult::Authenticate(x) => match x {
            Ok(url) => handle_output(
                format!(
                    "Open the following URL in a web browser to \
                     link your install ID with your asciinema.org user account:\
                     \n\n{}\n\n\
                     This will associate all recordings uploaded from this machine \
                     (past and future ones) to your account, and allow you to manage \
                     them (change title/theme, delete) at asciinema.org.",
                    url
                )
                .as_str(),
            ),
            Err(x) => handle_error(&x),
        },
        CommandResult::Concatenate(x) | CommandResult::Play(x) => match x {
            Ok(()) => 0,
            Err(x) => handle_error(&x),
        },
        CommandResult::Record(x) => match x {
            Ok(location) => {
                let location_output = match location {
                    RecordLocation::Local(f) => {
                        format!("asciicast saved to: {}", f.to_string_lossy())
                    }
                    RecordLocation::Remote(url) => format!("{}", url),
                };
                handle_output(location_output.as_str())
            }
            Err(x) => handle_error(&x),
        },
        CommandResult::Upload(x) => match x {
            Ok(url) => handle_output(format!("{}", url).as_str()),
            Err(x) => handle_error(&x),
        },
    })
}

fn handle_output(s: &str) -> i32 {
    println!("{}", s);
    // If we don't do this, the prompt when we exit is too far right. ¯\_(ツ)_/¯
    print!("{}", termion::cursor::Left(s.chars().count() as u16));
    0
}

fn handle_error(e: &Error) -> i32 {
    eprintln!("{}", e);
    1
}
