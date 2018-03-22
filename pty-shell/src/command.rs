use libc;
use std::{ffi, ptr};
use std::collections::HashMap;

pub fn exec<S: AsRef<str>>(shell: S) {
    let env: Option<HashMap<String, String>> = None;
    exec_with_env(shell, env)
}

pub fn exec_with_env<S: AsRef<str>>(shell: S, env: Option<HashMap<String, String>>) {
    // The command we are running.
    let cmd = ffi::CString::new(shell.as_ref()).unwrap();

    // The arguments to the command.
    let mut args: Vec<*const libc::c_char> = Vec::with_capacity(1);
    args.push(cmd.as_ptr());
    args.push(ptr::null());

    // The environment we are running in.
    let v = env.unwrap_or(HashMap::new());
    // We need to save the `CString`s in this scope, otherwise the pointers will be drefed
    // and all hell breaks loose.
    let cstring_vars: Vec<ffi::CString> = v.iter()
        .map(|(name, value)| ffi::CString::new(format!("{}={}", name, value)).unwrap())
        .collect();
    let mut env_vars: Vec<*const libc::c_char> = cstring_vars.iter().map(|c| c.as_ptr()).collect();
    env_vars.push(ptr::null());

    unsafe {
        libc::execve(cmd.as_ptr(), args.as_ptr(), env_vars.as_ptr());
    }
}
