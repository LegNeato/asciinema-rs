extern crate pty_shell;

use self::pty_shell::*;

struct TestHandler;

impl PtyHandler for TestHandler {
    fn output(&mut self, data: &[u8]) {
        assert!(data.len() != 0);
    }
}

#[test]
fn it_can_hook_stdout_with_handler() {
    let child = tty::Fork::from_ptmx().unwrap();
    restore_termios();

    child.proxy(TestHandler).unwrap();
    child.exec("pwd").unwrap();

    assert!(child.wait().is_ok());
}
