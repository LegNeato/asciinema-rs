use pty_shell::{restore_termios, tty, PtyCallback, PtyShell};

#[test]
fn it_can_hook_stdout_with_callback() {
    let child = tty::Fork::from_ptmx().unwrap();
    restore_termios();

    child
        .proxy(
            PtyCallback::default()
                .output(|data| assert!(data.len() != 0))
                .build(),
        )
        .unwrap();
    child.exec("pwd").unwrap();

    assert!(child.wait().is_ok());
}
