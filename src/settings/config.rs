use failure::Error;
use std::path::PathBuf;
use std::env;
use config::{Config, ConfigError, File};
use super::{ApiSettings, RecordSettings};

// Newtypes so we can keep the containing dir vs config file straight.
#[derive(Debug)]
struct AsciinemaConfigDir(PathBuf);
#[derive(Debug)]
struct AsciinemaConfigFile(PathBuf);

#[derive(Debug, Fail)]
enum ConfigFailure {
    #[fail(display = "unable to find home directory")] NoHome {},
    #[fail(display = "{}", _0)] ConfigFileParsingError(#[cause] ConfigError),
}

#[derive(Debug, Deserialize)]
pub struct AsciinemaConfig {
    pub api: Option<ApiSettings>,
    pub record: Option<RecordSettings>,
    // TODO: `play` settings
}

impl AsciinemaConfig {
    pub fn new() -> Result<Self, Error> {
        let location: AsciinemaConfigFile = get_config_file()?;

        let mut s = Config::new();
        s.merge(File::from(location.0).required(false))?;

        match s.try_into() {
            Ok(x) => Ok(x),
            Err(e) => Err(ConfigFailure::ConfigFileParsingError(e))?,
        }
    }
}

/// Finds the location in home directory to write configuration to using
/// strategy from https://asciinema.org/docs/config.
fn get_config_dir() -> Result<AsciinemaConfigDir, Error> {
    // If config location explictly set, use the value.
    if let Ok(config_home) = env::var("ASCIINEMA_CONFIG_HOME") {
        return Ok(AsciinemaConfigDir(PathBuf::from(config_home)));
    }

    // If XDG home is set, use that.
    if let Ok(xdg_home) = env::var("XDG_CONFIG_HOME") {
        return Ok(AsciinemaConfigDir(PathBuf::from(format!(
            "{}/asciinema",
            xdg_home
        ))));
    }

    // Otherwise fall back to `.config` in $HOME.
    if let Ok(home) = env::var("HOME") {
        return Ok(AsciinemaConfigDir(PathBuf::from(format!(
            "{}/.config/asciinema",
            home
        ))));
    }

    // Should only get here if `$HOME` isn't set.
    Err(ConfigFailure::NoHome {})?
}

fn get_config_file() -> Result<AsciinemaConfigFile, Error> {
    let mut location: AsciinemaConfigDir = get_config_dir()?;
    location.0.push("config");
    Ok(AsciinemaConfigFile(location.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_env() {
        env::set_var("ASCIINEMA_CONFIG_HOME", "/ach");
        env::set_var("XDG_CONFIG_HOME", "/xdg");
        env::set_var("HOME", "/home");
    }

    #[test]
    fn asciinema_config_home() {
        test_env();
        assert_eq!(get_config_dir().unwrap().0, PathBuf::from("/ach"));
    }

    #[test]
    fn xdg_config_home() {
        test_env();
        env::remove_var("ASCIINEMA_CONFIG_HOME");
        assert_eq!(get_config_dir().unwrap().0, PathBuf::from("/xdg/asciinema"));
    }

    #[test]
    fn home() {
        test_env();
        env::remove_var("ASCIINEMA_CONFIG_HOME");
        env::remove_var("XDG_CONFIG_HOME");
        assert_eq!(
            get_config_dir().unwrap().0,
            PathBuf::from("/home/.config/asciinema")
        );
    }

    #[test]
    fn no_home() {
        test_env();
        env::remove_var("ASCIINEMA_CONFIG_HOME");
        env::remove_var("XDG_CONFIG_HOME");
        env::remove_var("HOME");
        // TODO: Figure out a better way to check this.
        assert_eq!(
            format!("{}", get_config_dir().unwrap_err()),
            format!("{}", ConfigFailure::NoHome {}),
        );
    }
}
