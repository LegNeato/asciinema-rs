use std::path::PathBuf;
use structopt::StructOpt;
use url::Url;
use url_serde;
use failure::Error;
use uuid::Uuid;

mod cli;
mod config;
pub mod install;

use self::config::AsciinemaConfig;
use self::cli::CommandLine;

pub enum Action {
    Authenticate,
    Concatenate,
    Play,
    Record,
    Upload,
}

pub struct Settings {
    pub action: Action,
    pub api_url: Url,
    pub authenticate: Option<AuthenticateSettings>,
    pub concatenate: Option<ConcatenateSettings>,
    pub play: Option<PlaySettings>,
    pub record: Option<RecordSettings>,
    pub upload: Option<UploadSettings>,
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
            CommandLine::Authenticate { 0: x } => Ok(Settings {
                action: Action::Authenticate,
                api_url,
                authenticate: Some(AuthenticateSettings { ..x }),
                concatenate: None,
                play: None,
                record: None,
                upload: None,
            }),
            CommandLine::Concatenate { 0: x } => Ok(Settings {
                action: Action::Concatenate,
                api_url,
                authenticate: None,
                concatenate: Some(ConcatenateSettings { ..x }),
                play: None,
                record: None,
                upload: None,
            }),
            CommandLine::Play { 0: x } => Ok(Settings {
                action: Action::Play,
                api_url,
                authenticate: None,
                concatenate: None,
                play: Some(PlaySettings { ..x }),
                record: None,
                upload: None,
            }),
            CommandLine::Record { 0: x } => Ok(Settings {
                action: Action::Record,
                api_url,
                authenticate: None,
                concatenate: None,
                play: None,
                record: Some(RecordSettings { ..x }),
                upload: None,
            }),
            CommandLine::Upload { 0: x } => Ok(Settings {
                action: Action::Upload,
                api_url,
                authenticate: None,
                concatenate: None,
                play: None,
                record: None,
                upload: Some(UploadSettings { ..x }),
            }),
        }
    }
}

#[derive(StructOpt, Clone, Debug, Deserialize)]
pub struct PlaySettings {
    /// Limit replayed terminal inactivity to max seconds
    #[structopt(short = "i", long = "idle-time-limit")]
    pub idle_time_limit: Option<f64>,
    /// Playback speed
    #[structopt(short = "s", long = "speed")]
    pub speed: Option<f64>,
    /// Location can be either local recording or remote recording
    #[structopt(name = "LOCATION", parse(from_os_str))]
    pub location: PathBuf,
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
    #[structopt(long = "raw", raw(requires = r#""FILE""#))]
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

#[derive(StructOpt, Clone, Debug, Deserialize)]
pub struct AuthenticateSettings {
    /// An existing UUIDv4 install id to use
    #[structopt(short = "i", long = "install-id")]
    pub install_id: Option<Uuid>,
}

#[derive(StructOpt, Clone, Debug, Deserialize)]
pub struct UploadSettings {
    /// Filename/path of local recording
    #[structopt(name = "FILE", parse(from_os_str))]
    pub file: PathBuf,
}

#[derive(StructOpt, Clone, Debug, Deserialize)]
pub struct ConcatenateSettings {
    /// Location can be either local recording or remote recording
    #[structopt(name = "LOCATION", parse(from_os_str))]
    pub location: PathBuf,
}
