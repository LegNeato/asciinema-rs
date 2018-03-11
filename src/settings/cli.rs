use super::RecordSettings;

#[derive(StructOpt, Debug)]
#[structopt(name = "asciinema", author = "")]
/// Record and share your terminal sessions, the right way.
pub enum CommandLine {
    /// Record terminal session
    #[structopt(name = "rec")]
    Record(RecordSettings),
}
