use std::{ffi::OsString, os::unix::ffi::OsStringExt};

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

fn main() {
    let flags = flags::Cmd::from_vec(vec![
        OsString::from_vec(vec![254].into()),
        OsString::from_vec(vec![255].into()),
        "utf8".into(),
    ]);

    eprintln!("flags = {:?}", flags);
}
