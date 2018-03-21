# [asciinema-rs][self]

[![Build Status](https://travis-ci.org/LegNeato/asciinema-rs.svg?branch=master)](https://travis-ci.org/LegNeato/asciinema-rs)

A reimplementation of the [asciinema][asciinema] command line program written in
Rust.

### Installation

Prebuilt `asciinema` binaries can be downloaded from [GitHub releases](https://github.com/LegNeato/asciinema-rs/releases). There are no
dependencies and the binary can be run directly once downloaded.

(`asciinema` is not available on [crates.io](https://crates.io) due to https://github.com/rust-lang/cargo/issues/1565).

### Usage

This program intends be a drop-in replacement for the official
[asciinema][asciinema] python [client program][original]. Accordingly, the
[official documentation](https://asciinema.org/docs/getting-started) serves
as documentation for this version as well. If you find behavior differences, please file an issue.

```sh
# Record terminal and upload it to asciinema.org:
asciinema rec

# Record terminal to local file:
asciinema rec demo.cast

# Record terminal and upload it to asciinema.org, specifying title:
asciinema rec -t "My git tutorial"

# Record terminal to local file, limiting idle time to max 2.5 sec:
asciinema rec -i 2.5 demo.cast
```

### Example

Below is an example recording where the program records itself.

<a href="https://asciinema.org/a/CYnuc8LuJ6WYSc9oDpiF1GDav"><img src="https://asciinema.org/a/CYnuc8LuJ6WYSc9oDpiF1GDav.png" width="50%"></a>

### Feature Parity

We are not yet at 100% parity with the official binary. This project welcomes
contributors and is a great project for Rust beginners. **Please contribute!**

#### Record (`asciinema rec`)

* [x] Record `stdout`
* [ ] [Record `stdin` via `--stdin`][issue.4]
* [x] Upload to remote server
* [x] Save to local file
* [ ] [Prompt for where to save after recording][issue.5]
* [x] Append to output via `--append`
* [x] Save only raw stdout output via `--raw`
* [x] `--overwrite`
* [ ] [Run a command via `--command`][issue.3]
* [ ] [By default capture `SHELL` and `TERM` environment variables][issue.7]
* [ ] [Specify environment variables to capture via `--env`][issue.8]
* [x] Set a title via `--title`
* [x] Set an idle limit via `--idle-time-limit`
* [ ] [Answer yes to all prompts via `--yes`][issue.9]
* [ ] [Quiet output via `--quiet`][issue.10]

#### Auth (`asciinema auth`)

* [x] Authenticate via an install-id

#### Upload (`asciinema upload`)

* [x] Upload saved asciicast session

#### Play (`asciinema play`)

* [ ] [Implement `play` command][issue.1]

#### Cat (`asciinema cat`)

* [x] Support for local files
* [x] Support for remote files

### License

`asciinema` is licensed under either of the following, at your option:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

Note that the reference [python implementation][original] is licensed under [GPLv3](https://github.com/asciinema/asciinema/blob/develop/LICENSE). This program is developed without looking at or using any of the code.

[self]: https://github.com/LegNeato/asciinema-rs
[asciinema]: https://asciinema.org
[original]: https://github.com/asciinema/asciinema
[issue.1]: https://github.com/LegNeato/asciinema-rs/issues/1
[issue.2]: https://github.com/LegNeato/asciinema-rs/issues/2
[issue.3]: https://github.com/LegNeato/asciinema-rs/issues/3
[issue.4]: https://github.com/LegNeato/asciinema-rs/issues/4
[issue.5]: https://github.com/LegNeato/asciinema-rs/issues/5
[issue.6]: https://github.com/LegNeato/asciinema-rs/issues/6
[issue.7]: https://github.com/LegNeato/asciinema-rs/issues/7
[issue.8]: https://github.com/LegNeato/asciinema-rs/issues/8
[issue.9]: https://github.com/LegNeato/asciinema-rs/issues/9
[issue.10]: https://github.com/LegNeato/asciinema-rs/issues/10
