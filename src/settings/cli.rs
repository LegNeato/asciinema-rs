#[cfg(feature = "output_gif")]
use super::ConvertSettings;
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
    #[structopt(raw(visible_alias = r#""auth""#))]
    Authenticate(AuthenticateSettings),
    /// Print full output of terminal session
    #[structopt(name = "concatenate")]
    #[structopt(raw(visible_alias = r#""cat""#))]
    Concatenate(ConcatenateSettings),
    /// Convert an asciicast to a different format
    #[cfg(feature = "output_gif")]
    #[structopt(name = "convert")]
    Convert(ConvertSettings),
    /// Replay recorded asciicast in a terminal
    #[structopt(name = "play")]
    Play(PlaySettings),
    /// Record terminal session
    #[structopt(name = "record")]
    #[structopt(raw(visible_alias = r#""rec""#))]
    Record(RecordSettings),
    /// Upload locally saved terminal session to asciinema.org
    #[structopt(name = "upload")]
    #[structopt(raw(visible_alias = r#""up""#))]
    Upload(UploadSettings),
}
