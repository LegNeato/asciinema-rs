use api::Api;
use failure::Error;
use os_type;
use reqwest;
use reqwest::header::{HeaderMap, LOCATION, USER_AGENT};
use std::env;
use std::path::PathBuf;
use url::Url;
use uuid::Uuid;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Fail)]
enum UploadFailure {
    #[fail(display = "server failed to return a URL")]
    InvalidResponseLocation {},
}

fn user_agent_string() -> String {
    let os = os_type::current_platform();
    format!("asciinema-rs/{} {:?}/{}", VERSION, os.os_type, os.version)
}

fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        user_agent_string().parse().expect("valid user agent"),
    );
    headers
}

#[derive(Default, Builder, Debug)]
#[builder(setter(into))]
pub struct Upload {
    api: Api,
    install_id: Uuid,
    #[builder(default = "self.get_current_user()")]
    user: String,
}

impl UploadBuilder {
    // Private helper method with access to the builder struct.
    fn get_current_user(&self) -> String {
        env::var("USER").unwrap_or_else(|_| "Unknown".to_string())
    }
}

impl Upload {
    pub fn upload_file(self, file: PathBuf) -> Result<Url, Error> {
        let files = reqwest::multipart::Form::new().file("asciicast", file)?;

        let response = reqwest::Client::new()
            .post(self.api.upload_url())
            .headers(construct_headers())
            .multipart(files)
            .basic_auth(self.user, Some(self.install_id.hyphenated().to_string()))
            .send()?
            .error_for_status()?;

        let location = response
            .headers()
            .get(LOCATION)
            .unwrap_or(Err(UploadFailure::InvalidResponseLocation {})?)
            .to_str()
            .map(|loc| response.url().join(loc))?;

        // TODO: Handle `Warning` header.
        // TODO: map status codes to app-specific failure messages.

        match location {
            Ok(loc) => Ok(loc),
            Err(e) => Err(e)?,
        }
    }
}
