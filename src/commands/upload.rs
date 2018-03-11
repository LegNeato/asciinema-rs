use failure::Error;
use url;
use url::Url;
use settings::UploadSettings;
use settings::install::InstallInfo;
use reqwest;
use reqwest::header::{Headers, Location, UserAgent};
use os_type;
use std::env;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Fail)]
enum UploadFailure {
    #[fail(display = "server failed to return a URL")] InvalidResponseLocation {},
}

fn get_upload_url(base_url: Url) -> Result<Url, url::ParseError> {
    base_url.join("/api/asciicasts")
}

fn user_agent_string() -> String {
    let os = os_type::current_platform();
    format!("asciinema-rs/{} {:?}/{}", VERSION, os.os_type, os.version)
}

fn construct_headers() -> Headers {
    let mut headers = Headers::new();
    headers.set(UserAgent::new(user_agent_string()));
    headers
}

fn get_current_user() -> String {
    env::var("USER").unwrap_or("Unknown".to_string())
}

pub fn go(settings: UploadSettings, api_url: Url) -> Result<Url, Error> {
    // Load install id from a file or generate a new one.
    // Note: the reference python version doesn't fail when
    // there is no existing install id, so we don't either.
    let install_info = InstallInfo::new()?;

    let id = install_info.id;

    // Persist the install id to a file.
    // Note: the reference python version persists the install id
    // so we do as well.
    if !install_info.is_saved {
        install_info.save()?;
    }

    let files = reqwest::multipart::Form::new().file("asciicast", settings.file)?;

    let response = reqwest::Client::new()
        .post(get_upload_url(api_url.clone())?)
        .headers(construct_headers())
        .multipart(files)
        .basic_auth(get_current_user(), Some(id.hyphenated().to_string()))
        .send()?
        .error_for_status()?;

    let location = response
        .headers()
        .get::<Location>()
        .map(|loc| response.url().join(loc));

    // TODO: Handle `Warning` header.
    // TODO: map status codes to app-specific failure messages.

    match location {
        Some(Ok(loc)) => Ok(loc),
        Some(Err(e)) => Err(e)?,
        None => Err(UploadFailure::InvalidResponseLocation {})?,
    }
}
