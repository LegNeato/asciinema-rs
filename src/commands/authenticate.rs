use failure::Error;
use url;
use url::Url;
use settings::AuthenticateSettings;
use settings::install::InstallInfo;
use uuid::Uuid;

fn get_authentication_url(base_url: Url, uuid: Uuid) -> Result<Url, url::ParseError> {
    base_url
        .join("/connect/")?
        .join(&uuid.hyphenated().to_string())
}

pub fn go(settings: AuthenticateSettings, api_url: Url) -> Result<Url, Error> {
    // Load install id from a file or generate a new one.
    let mut install_info = InstallInfo::new()?;

    // Use and save any manually passed in install id.
    if let Some(new_id) = settings.install_id {
        install_info.id = new_id;
        install_info.is_saved = false;
    }

    let id = install_info.id;

    // Persist the install id to a file.
    if !install_info.is_saved {
        install_info.save()?;
    }

    Ok(get_authentication_url(api_url, id)?)
}
