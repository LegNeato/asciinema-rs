# [asciinema]

[![Build Status](https://travis-ci.org/LegNeato/asciinema-rs.svg?branch=master)](https://travis-ci.org/LegNeato/asciinema-rs)

A reimplementation of the [asciinema][asciinema] command line program written in
Rust.

### Installation

`asciinema` is available on [crates.io](https://crates.io/crates/asciinema) and can be installed via Cargo:

```sh
cargo install asciinema
```

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

### Feature Parity

We are not yet at 100% parity with the official binary. This project welcomes
contributors and is a great project for Rust beginners. **Please contribute!**

#### Record (`asciinema rec`)

* [x] Record `stdout`
* [ ] Record `stdin` via `--stdin`
* [x] Upload to remote server
* [x] Save to local file
* [ ] Prompt for where to save after recording
* [x] Append to output via `--append`
* [ ] Save only raw stdout output via `--raw`
* [x] `--overwrite`
* [ ] Run a command via `--command`
* [ ] By default capture `SHELL` and `TERM` environment variables
* [ ] Specify environment variables to capture via `--env`
* [x] Set a title via `--title`
* [x] Set an idle limit via `--idle-time-limit`
* [ ] Answer yes to all prompts via `--yes`
* [ ] Quiet output via `--quiet`

#### Auth (`asciinema auth`)

* [x] Authenticate via an install-id.

#### Upload (`asciinema upload`)

* [x] Upload saved asciicast session.

#### Play (`asciinema play`)

* [ ] Implement `play` command

#### Cat (`asciinema cat`)

* [ ] Implement `cat` command

### License

`asciinema` is licensed under either of the following, at your option:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

Note that the reference [python implementation][original] is licensed under [GPLv3](https://github.com/asciinema/asciinema/blob/develop/LICENSE). This program is developed without looking at or using any of the code.

[asciinema]: https://asciinema.org
[original]: https://github.com/asciinema/asciinema
