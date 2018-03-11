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
    // Note: the reference python version doesn't fail when
    // there is no existing install id, so we don't either.
    let install_info = InstallInfo::new()?;

    // Persist the install id to a file.
    // Note: the reference python version persists the install id
    // so we do as well.
    if !install_info.is_saved {
        install_info.save()?;
    }

    Ok(get_connect_url(api_url, install_info.id)?)
}
