## [[Unreleased] - ReleaseDate](https://github.com/LegNeato/asciinema-rs/releases/tag/vUnreleased)

### Added

* Additional command aliases are now supported, allowing brevity or verbosity at your discretion. ([#12](https://github.com/LegNeato/asciinema-rs/issues/12)).

  * `auth`: aliased to `authenticate` and `a`
  * `cat`: aliased to `concatenate` and `c`
  * `rec`: aliased to `record` and `r`
  * `upload`: aliased to `up` and `u`

* The `--raw` option is now supported when recording ([#19](https://github.com/LegNeato/asciinema-rs/pull/19)).

## [[0.2.0] - 2018-03-16](https://github.com/LegNeato/asciinema-rs/releases/tag/v0.2.0)

### Added

* Support for the `cat` subcommand ([#18](https://github.com/LegNeato/asciinema-rs/pull/18)).

### Fixed

* The `url` value set in the `[api]` section of the [config](https://asciinema.org/docs/config) no longer needs a trailing slash to behave correctly.

## [[0.1.0] - 2018-03-12](https://github.com/LegNeato/asciinema-rs/releases/tag/v0.1.0)

* Initial release
