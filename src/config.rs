use failure::Error;
use std::path::PathBuf;
use std::env;

#[derive(Debug, Fail)]
enum ConfigFailure {
    #[fail(display = "unable to find home directory")] NoHome {},
}

/// Finds the location in home directory to write configuration to using
/// strategy from https://asciinema.org/docs/config.
pub fn get_location() -> Result<PathBuf, Error> {

    // If config location explictly set, use the value.
    if let Ok(config_home) = env::var("ASCIINEMA_CONFIG_HOME") {
        return Ok(PathBuf::from(config_home));
    }

    // If XDG home is set, use that.
    if let Ok(xdg_home) = env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(format!("{}/asciinema", xdg_home)))
    }

    // Otherwise fall back to `.config` in $HOME.
    if let Ok(home) = env::var("HOME") {
        return Ok(PathBuf::from(format!("{}/.config/asciinema", home)))
    }

    // Should only get here if `$HOME` isn't set.
    Err(ConfigFailure::NoHome {})?
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
        assert_eq!(get_location().unwrap(), PathBuf::from("/ach"));
    }

    #[test]
    fn xdg_config_home() {
        test_env();
        env::remove_var("ASCIINEMA_CONFIG_HOME");
        assert_eq!(get_location().unwrap(), PathBuf::from("/xdg/asciinema"));
    }

    #[test]
    fn home() {
        test_env();
        env::remove_var("ASCIINEMA_CONFIG_HOME");
        env::remove_var("XDG_CONFIG_HOME");
        assert_eq!(get_location().unwrap(), PathBuf::from("/home/.config/asciinema"));
    }

    #[test]
    fn no_home() {
        test_env();
        env::remove_var("ASCIINEMA_CONFIG_HOME");
        env::remove_var("XDG_CONFIG_HOME");
        env::remove_var("HOME");
        // TODO: Figure out a better way to check this.
        assert_eq!(
            format!("{}", get_location().unwrap_err()),
            format!("{}", ConfigFailure::NoHome {}),
        );
    }
}
