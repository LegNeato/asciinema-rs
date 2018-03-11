use super::{AuthenticateSettings, RecordSettings, UploadSettings};

#[derive(StructOpt, Debug)]
#[structopt(name = "asciinema", author = "")]
/// Record and share your terminal sessions, the right way.
pub enum CommandLine {
    /// Manage recordings on asciinema.org account
    #[structopt(name = "auth")]
    Authenticate(AuthenticateSettings),
    /// Record terminal session
    #[structopt(name = "rec")]
    Record(RecordSettings),
    /// Upload locally saved terminal session to asciinema.org
    #[structopt(name = "upload")]
    Upload(UploadSettings),
}
