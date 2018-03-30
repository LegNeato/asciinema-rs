use super::{AuthenticateSettings, ConcatenateSettings, PlaySettings, RecordSettings,
            UploadSettings};
use structopt::clap::AppSettings;

#[derive(StructOpt, Debug)]
#[structopt(name = "asciinema", author = "")]
#[structopt(raw(global_settings = "&[AppSettings::VersionlessSubcommands, AppSettings::InferSubcommands]"))]
/// Record and share your terminal sessions, the right way.
pub enum CommandLine {
    /// Manage recordings on asciinema.org account
    #[structopt(name = "authenticate")]
    Authenticate(AuthenticateSettings),
    /// Print full output of terminal session
    #[structopt(name = "concatenate")]
    #[structopt(raw(aliases = r#"&["cat"]"#))]
    Concatenate(ConcatenateSettings),
    /// Replay recorded asciicast in a terminal
    #[structopt(name = "play")]
    Play(PlaySettings),
    /// Record terminal session
    #[structopt(name = "record")]
    Record(RecordSettings),
    /// Upload locally saved terminal session to asciinema.org
    #[structopt(name = "upload")]
    Upload(UploadSettings),
}
