extern crate pty_shell;

use pty_shell::*;

struct Shell;
impl PtyHandler for Shell {
    fn input(&mut self, input: &[u8]) {
        // do something with input
    }

    fn output(&mut self, output: &[u8]) {
        // do something with output
    }

    fn resize(&mut self, winsize: &winsize::Winsize) {
        // do something with winsize
    }

    fn shutdown(&mut self) {
        // prepare for shutdown
    }
}

fn main() {
    let child = tty::Fork::from_ptmx().unwrap();

    child.exec("bash");
    child.proxy(Shell);
    child.wait();
}
