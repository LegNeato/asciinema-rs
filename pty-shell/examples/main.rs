use pty_shell::*;

struct Shell;
impl PtyHandler for Shell {
    fn input(&mut self, _input: &[u8]) {
        // do something with input
    }

    fn output(&mut self, _output: &[u8]) {
        // do something with output
    }

    fn resize(&mut self, _winsize: &winsize::Winsize) {
        // do something with winsize
    }

    fn shutdown(&mut self) {
        // prepare for shutdown
    }
}

fn main() {
    let child = tty::Fork::from_ptmx().unwrap();

    child.exec("bash").unwrap();
    child.proxy(Shell).unwrap();
    child.wait().unwrap();
}
