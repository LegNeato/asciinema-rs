use crate::api::Api;
use crate::settings::install::InstallInfo;
use crate::settings::AuthenticateSettings;
use failure::Error;
use url;
use url::Url;
use uuid::Uuid;

fn make_authentication_url(url: &Url, uuid: Uuid) -> Result<Url, url::ParseError> {
    url.join(&uuid.hyphenated().to_string())
}

pub fn go(settings: &AuthenticateSettings, api: Api) -> Result<Url, Error> {
    // Load install id from a file or generate a new one.
    let mut install_info = InstallInfo::new()?;

    // Use and save any manually passed in install id.
    if let Some(new_id) = settings.install_id {
        install_info.id = new_id;
        install_info.save()?;
    }

    Ok(make_authentication_url(
        &api.authentication_url(),
        install_info.id,
    )?)
}
