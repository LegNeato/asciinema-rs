## master

## [[0.3.0] - 2018-03-21](https://github.com/LegNeato/asciinema-rs/releases/tag/v0.3.0)

### Added

* Additional command aliases are now supported, allowing brevity or verbosity at your discretion ([#12](https://github.com/LegNeato/asciinema-rs/issues/12)).

  * `auth`: aliased to `authenticate` and `a`
  * `cat`: aliased to `concatenate` and `c`
  * `rec`: aliased to `record` and `r`
  * `upload`: aliased to `up` and `u`

* The `--raw` option is now supported when recording ([#19](https://github.com/LegNeato/asciinema-rs/pull/19)).

* The environment variable `ASCIINEMA_REC` is set to `1` in recorded process
  environment variables ([#21](https://github.com/LegNeato/asciinema-rs/issues/21)).

  As mentioned in the [official client's usage](https://asciinema.org/docs/usage), this can be used by your shell's config file (`.bashrc`, `.zshrc`) to alter the prompt or play a sound when the shell is being recorded.

* `SHELL` and `TERM` environment variables are captured by default when recording ([#7](https://github.com/LegNeato/asciinema-rs/issues/7)).

### Fixed

* All existing environment variables are now set in the recorded process as well.

  Previously the recorded process did not inherit anything from the parent and instead had an empty environment.

## [[0.2.0] - 2018-03-16](https://github.com/LegNeato/asciinema-rs/releases/tag/v0.2.0)

### Added

* Support for the `cat` subcommand ([#18](https://github.com/LegNeato/asciinema-rs/pull/18)).

### Fixed

* The `url` value set in the `[api]` section of the [config](https://asciinema.org/docs/config) no longer needs a trailing slash to behave correctly.

## [[0.1.0] - 2018-03-12](https://github.com/LegNeato/asciinema-rs/releases/tag/v0.1.0)

* Initial release
