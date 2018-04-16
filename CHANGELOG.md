## master

### Added

* The `--stdin` option is now supported when recording.

  Stdin recording allows for capturing of all characters typed in by the user
  in the currently recorded shell. This may be used by a player
  (e.g. asciinema-player) to display pressed keys. Because **it's basically a
  key-logger** (scoped to a single shell instance), it’s disabled by default,
  and has to be explicitly enabled via the `--stdin` option.

### Fixed

* The OpenSSL certificate error thrown when uploading to asciinema.org
  using the pre-compiled Linux binary has been fixed ([#40](https://github.com/LegNeato/asciinema-rs/issues/40)).

## [[0.4.0] - 2018-03-26](https://github.com/LegNeato/asciinema-rs/releases/tag/v0.4.0)

### Added

* The `play` command is now supported for local and remote files ([#32](https://github.com/LegNeato/asciinema-rs/pull/32)).

  Note: support for interactive features such as pausing playback will be added
  in the future.

* Playback speed can be adjusted via `asciinema play --speed`. A value of `2`
  would make playback twice as fast as realtime. A value of `0.5` would make
  playback half as fast as realtime.

* Playback idle time can be adjusted via `asciinema play --idle-time-limit`. A
  value of `2` would limit the playback idle time to a maximum of 2 seconds.

* The cursor can be hidden during playback via `asciinema play --hide-cursor`.

### Fixed

* When a local recording output file is specified, events are now written
  to the file in realtime ([#17](https://github.com/LegNeato/asciinema-rs/issues/17)).

  Previously the events were queued and written to the
  output file at the end of the recording session.

  Note: Due to this change terminal-to-terminal streaming is now possible,
  as mentioned in this [asciinema blog post](http://blog.asciinema.org/post/two-point-o/).

  Locally via a Unix pipe:

  ```bash
  mkfifo /tmp/demo.pipe

  # viewing terminal
  asciinema play /tmp/demo.pipe

  # recording terminal
  asciinema rec /tmp/demo.pipe
  ```

  Over the network via `netcat`:

  ```bash
  # viewing terminal (hostname: node123)
  asciinema play <(nc -l localhost 9999)

  # recording terminal
  asciinema rec >(nc node123 9999)
  ```

* When appending to a recording via `asciinema rec --append`,
  the header is no longer written.

  Previously the header was written regardless.

* The terminal's width and height is correctly determined when recording during
  another recording.

  Previously the interior's recording failed to detect the terminal size and instead set the width and height to zero.

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
