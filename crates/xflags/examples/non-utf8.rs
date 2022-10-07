use std::ffi::OsString;

mod flags {
    use std::{ffi::OsString, path::PathBuf};

    xflags::xflags! {
        cmd Cmd
            required a: OsString
            required b: PathBuf
            required c: String
        {
        }
    }
}

#[cfg(unix)]
fn main() {
    use std::os::unix::ffi::OsStringExt;

    let flags = flags::Cmd::from_vec(vec![
        OsString::from_vec(vec![254].into()),
        OsString::from_vec(vec![255].into()),
        "utf8".into(),
    ]);

    eprintln!("flags = {:?}", flags);
}

#[cfg(windows)]
fn main() {
    use std::os::windows::ffi::OsStringExt;

    let flags = flags::Cmd::from_vec(vec![
        OsString::from_wide(&[0xD800]),
        OsString::from_wide(&[0xDC00]),
        "utf8".into(),
    ]);

    eprintln!("flags = {:?}", flags);
}

#[cfg(not(any(unix, windows)))]
fn main() {}
