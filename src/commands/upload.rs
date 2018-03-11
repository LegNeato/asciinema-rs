use failure::Error;
use url;
use url::Url;
use settings::UploadSettings;
use settings::install::InstallInfo;
use uuid::Uuid;

fn get_connect_url(base_url: Url, uuid: Uuid) -> Result<Url, url::ParseError> {
    base_url
        .join("/connect/")?
        .join(&uuid.hyphenated().to_string())
}

pub fn go(settings: UploadSettings, api_url: Url) -> Result<Url, Error> {
    // Load install id from a file or generate a new one.
    let install_info = InstallInfo::new()?;

    Ok(get_connect_url(api_url, install_info.id)?)
}
