use std::path::PathBuf;
use structopt::StructOpt;
use url::Url;
use url_serde;
use failure::Error;

mod cli;
mod config;

use self::config::AsciinemaConfig;
use self::cli::CommandLine;

pub enum Action {
    Record,
}

pub struct Settings {
    pub action: Action,
    pub api_url: Url,
    pub record: RecordSettings,
}

impl Settings {
    pub fn new() -> Result<Self, Error> {
        // Load saved config.
        let config = AsciinemaConfig::new()?;

        // Api settings cannot be entered on the command line, only
        // via the config file.
        let api_url = config
            .api
            .and_then(|x| x.url)
            .unwrap_or(Url::parse("https://asciinema.org")?);

        // Get settings to override from the command line.
        match CommandLine::from_args() {
            CommandLine::Record { 0: x } => Ok(Settings {
                action: Action::Record,
                api_url,
                record: RecordSettings { ..x },
            }),
        }
    }
}

#[derive(StructOpt, Clone, Debug, Deserialize)]
#[structopt(name = "rec")]
pub struct RecordSettings {
    /// Title of the asciicast
    #[structopt(short = "t", long = "title")]
    pub title: Option<String>,
    // TODO: command
    /// Limit recorded idle time to given number of seconds
    #[structopt(name = "IDLE_TIME_LIMIT", short = "i", long = "idle-time-limit")]
    pub idle_time_limit: Option<f64>,
    /// Answer "yes" to all prompts (e.g. upload confirmation)
    #[structopt(short = "y", long = "yes")]
    pub force_yes: bool,
    /// Overwrite the file if it already exists
    #[structopt(long = "overwrite")]
    pub overwrite: bool,
    /// Append to existing recording
    #[structopt(long = "append")]
    pub append: bool,
    /// Save only raw stdout output
    #[structopt(long = "raw")]
    pub raw: bool,
    /// Filename/path to save the recording to
    #[structopt(name = "FILE", parse(from_os_str))]
    pub file: Option<PathBuf>,
}

#[derive(StructOpt, Clone, Debug, Deserialize)]
pub struct ApiSettings {
    ///  API server URL.
    #[structopt(default_value = "https://asciinema.org")]
    #[serde(with = "url_serde")]
    pub url: Option<Url>,
}
