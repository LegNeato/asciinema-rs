use libc;
use std::{ffi, ptr};

pub fn exec<S: AsRef<str>>(shell: S) {
    let cmd = ffi::CString::new(shell.as_ref()).unwrap();
    let mut args: Vec<*const libc::c_char> = Vec::with_capacity(1);

    args.push(cmd.as_ptr());
    args.push(ptr::null());

    unsafe {
        libc::execvp(cmd.as_ptr(), args.as_mut_ptr());
    }
}
