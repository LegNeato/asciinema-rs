#[macro_export]
macro_rules! first_line_for_message {
    ($output_type:ident, $msg:expr,) => {{
        first_line_for_message!($output_type, $msg)
    }};
    ($output_type:ident, $msg:expr) => {{
        extern crate tempfile;
        use std::io::BufRead;
        use std::io::BufReader;
        use tempfile::NamedTempFile;

        let mut tmpfile = NamedTempFile::new().expect("create temp file");
        let output = $output_type::new(tmpfile.reopen().unwrap());
        let channel = output.channel();
        let thread_handle = output.spawn();

        channel.send($msg).expect("send message");
        channel.send(Msg::Finish).expect("send finish");
        thread_handle
            .join()
            .expect("join thread")
            .expect("no thread error");

        let f = BufReader::new(&tmpfile);
        f.lines().next()
    }};
}
