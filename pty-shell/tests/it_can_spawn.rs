use pty_shell::{restore_termios, tty, PtyShell};

#[test]
fn it_can_spawn() {
    let child = tty::Fork::from_ptmx().unwrap();
    restore_termios();

    child.exec("pwd").unwrap();

    assert!(child.wait().is_ok());
}
