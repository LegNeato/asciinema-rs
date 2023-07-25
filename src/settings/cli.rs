use super::{
    AuthenticateSettings, ConcatenateSettings, PlaySettings, RecordSettings, UploadSettings,
};
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "asciinema")]
#[structopt(
    global_settings = &[AppSettings::VersionlessSubcommands, AppSettings::InferSubcommands]
)]
/// Record and share your terminal sessions, the right way.
pub enum CommandLine {
    /// Manage recordings on asciinema.org account
    #[structopt(name = "authenticate")]
    #[structopt(alias = r#""auth""#)]
    Authenticate(AuthenticateSettings),
    /// Print full output of terminal session
    #[structopt(name = "concatenate")]
    #[structopt(alias = r#""cat""#)]
    Concatenate(ConcatenateSettings),
    /// Replay recorded asciicast in a terminal
    #[structopt(name = "play")]
    Play(PlaySettings),
    /// Record terminal session
    #[structopt(name = "record")]
    #[structopt(alias = r#""rec""#)]
    Record(RecordSettings),
    /// Upload locally saved terminal session to asciinema.org
    #[structopt(name = "upload")]
    #[structopt(alias = r#""up""#)]
    Upload(UploadSettings),
}
