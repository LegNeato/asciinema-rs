# pty-shell

An extension of pty crate.

https://speakerdeck.com/hibariya/control-a-shell-with-pty-shell

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
pty-shell = '0.2.0'
```

For example, add src/main.rs as following:

```rust
extern crate pty_shell;

use pty_shell::{winsize, PtyShell, PtyHandler};

struct Shell;
impl PtyHandler for Shell {
    fn input(&mut self, input: &[u8]) {
      /* do something with input */
    }

    fn output(&mut self, output: &[u8]) {
      /* do something with output */
    }

    fn resize(&mut self, winsize: &winsize::Winsize) {
      /* do something with winsize */
    }

    fn shutdown(&mut self) {
      /* prepare for shutdown */
    }
}

fn main() {
    let child = pty::fork().unwrap();

    child.exec("bash");
    child.proxy(Shell);
    child.wait();
}
```

### Callback Style

Use `pty_shell::PtyCallback`.

```rust
child.proxy(
    PtyCallback::new()
        .input(|input| { /* do something with input */ })
        .output(|output| { /* do something with output */ })
        .build()
    )
);
```

### Event types

* input
* output
* resize
* shutdown

## Contributing

1. Fork it ( https://github.com/hibariya/pty-shell/fork )
2. Create your feature branch (`git checkout -b my-new-feature`)
3. Commit your changes (`git commit -am 'Add some feature'`)
4. Push to the branch (`git push origin my-new-feature`)
5. Create a new Pull Request

## License

Copyright (c) 2015 Hika Hibariya

Distributed under the [MIT License](LICENSE.txt).
