#![allow(unused)]
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug)]
pub struct RepeatedPos {
    pub a: PathBuf,
    pub b: Option<u32>,
    pub c: Option<OsString>,
    pub rest: Vec<OsString>,
}

impl RepeatedPos {
    pub const HELP: &'static str = Self::HELP_;

    #[allow(dead_code)]
    pub fn from_env() -> xflags::Result<Self> {
        Self::from_env_()
    }

    #[allow(dead_code)]
    pub fn from_vec(args: Vec<std::ffi::OsString>) -> xflags::Result<Self> {
        Self::from_vec_(args)
    }
}

impl RepeatedPos {
    fn from_env_() -> xflags::Result<Self> {
        let mut p = xflags::rt::Parser::new_from_env();
        Self::parse_(&mut p)
    }
    fn from_vec_(args: Vec<std::ffi::OsString>) -> xflags::Result<Self> {
        let mut p = xflags::rt::Parser::new(args);
        Self::parse_(&mut p)
    }
}

impl RepeatedPos {
    fn parse_(p_: &mut xflags::rt::Parser) -> xflags::Result<Self> {
        let mut a = (false, Vec::new());
        let mut b = (false, Vec::new());
        let mut c = (false, Vec::new());
        let mut rest = (false, Vec::new());

        while let Some(arg_) = p_.pop_flag() {
            match arg_ {
                Ok(flag_) => match flag_.as_str() {
                    _ => return Err(p_.unexpected_flag(&flag_)),
                },
                Err(arg_) => {
                    if let (done_ @ false, buf_) = &mut a {
                        buf_.push(arg_.into());
                        *done_ = true;
                        continue;
                    }
                    if let (done_ @ false, buf_) = &mut b {
                        buf_.push(p_.value_from_str::<u32>("b", arg_)?);
                        *done_ = true;
                        continue;
                    }
                    if let (done_ @ false, buf_) = &mut c {
                        buf_.push(arg_.into());
                        *done_ = true;
                        continue;
                    }
                    if let (false, buf_) = &mut rest {
                        buf_.push(arg_.into());
                        continue;
                    }
                    return Err(p_.unexpected_arg(arg_));
                }
            }
        }
        Ok(Self {
            a: p_.required("a", a.1)?,
            b: p_.optional("b", b.1)?,
            c: p_.optional("c", c.1)?,
            rest: rest.1,
        })
    }
}
impl RepeatedPos {
    const HELP_: &'static str = "\
RepeatedPos

ARGS:
    <a>

    [b]

    [c]

    <rest>...
";
}
