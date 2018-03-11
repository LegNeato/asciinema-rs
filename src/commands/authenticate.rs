use failure::Error;
use url::Url;
use settings::AuthenticateSettings;

pub fn go(settings: AuthenticateSettings, api_url: Url) -> Result<Url, Error> {
    Ok(api_url)
}
