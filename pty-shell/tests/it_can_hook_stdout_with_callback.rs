extern crate pty_shell;

use self::pty_shell::*;

#[test]
fn it_can_hook_stdout_with_callback() {
    let child = tty::Fork::from_ptmx().unwrap();
    restore_termios();

    child.proxy(PtyCallback::new()
            .output(|data| assert!(data.len() != 0))
            .build())
        .unwrap();
    child.exec("pwd").unwrap();

    assert!(child.wait().is_ok());
}
