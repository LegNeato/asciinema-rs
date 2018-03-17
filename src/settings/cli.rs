use super::{AuthenticateSettings, ConcatenateSettings, RecordSettings, UploadSettings};

#[derive(StructOpt, Debug)]
#[structopt(name = "asciinema", author = "")]
/// Record and share your terminal sessions, the right way.
pub enum CommandLine {
    /// Manage recordings on asciinema.org account
    #[structopt(name = "auth")]
    Authenticate(AuthenticateSettings),
    /// Print full output of terminal session
    #[structopt(name = "cat")]
    Concatenate(ConcatenateSettings),
    /// Record terminal session
    #[structopt(name = "rec")]
    Record(RecordSettings),
    /// Upload locally saved terminal session to asciinema.org
    #[structopt(name = "upload")]
    Upload(UploadSettings),
}
