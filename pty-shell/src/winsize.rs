use libc;
use std::io;

// TODO: Support Windows.

#[repr(C)]
#[derive(PartialEq, Clone, Debug, Default)]
pub struct Winsize {
    pub ws_row: libc::c_ushort, // rows, in characters
    pub ws_col: libc::c_ushort, // columns, in characters
    pub ws_xpixel: libc::c_ushort, // horizontal size, pixels
    pub ws_ypixel: libc::c_ushort, // vertical size, pixels
}

pub fn from_fd(fd: libc::c_int) -> io::Result<Winsize> {
    let winsize = Winsize::default();

    unsafe {
        libc::ioctl(fd, libc::TIOCGWINSZ, &winsize);
    }

    Ok(winsize)
}

pub fn set(fd: libc::c_int, winsize: &Winsize) {
    unsafe {
        libc::ioctl(fd, libc::TIOCSWINSZ, winsize);
    }
}
