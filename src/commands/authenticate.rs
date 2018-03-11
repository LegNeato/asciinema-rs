use failure::Error;
use url::Url;
use settings::AuthenticateSettings;
use settings::install::InstallInfo;

pub fn go(
    _settings: AuthenticateSettings,
    install_info: InstallInfo,
    api_url: Url,
) -> Result<Url, Error> {
    let authed: InstallInfo = match install_info.id {
        Some(_) => install_info,
        None => install_info.generate(),
    };

    authed.clone().save()?;

    Ok(api_url.join("/connect/")?.join(&authed.id.unwrap().hyphenated().to_string())?)
}
