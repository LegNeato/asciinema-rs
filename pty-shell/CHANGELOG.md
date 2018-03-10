### 0.1.4

* [#4](https://github.com/hibariya/pty-shell/pull/4) Use derives Default and Clone for Winsize Structure

### 0.1.3

* Rename `PtyProxy` to `PtyShell`

### 0.1.2

* Change the argument type of the input/output handler (from `Vec<u8>` to `&[u8]`)
* Offer the `PtyCallback` as a builder
* Add `shutdown` handler
* Add `resize` handler

### 0.1.1

* Add `PtyCallback` struct for callback-style usage

### 0.1.0

* First implementation
