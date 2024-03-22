#[allow(unused)]
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug)]
pub struct Empty;

impl Empty {
    #[allow(dead_code)]
    pub fn from_env_or_exit() -> Self {
        Self::from_env_or_exit_()
    }

    #[allow(dead_code)]
    pub fn from_env() -> xflags::Result<Self> {
        Self::from_env_()
    }

    #[allow(dead_code)]
    pub fn from_vec(args: Vec<std::ffi::OsString>) -> xflags::Result<Self> {
        Self::from_vec_(args)
    }
}

impl Empty {
    fn from_env_or_exit_() -> Self {
        Self::from_env_().unwrap_or_else(|err| err.exit())
    }
    fn from_env_() -> xflags::Result<Self> {
        let mut p = xflags::rt::Parser::new_from_env();
        Self::parse_(&mut p)
    }
    fn from_vec_(args: Vec<std::ffi::OsString>) -> xflags::Result<Self> {
        let mut p = xflags::rt::Parser::new(args);
        Self::parse_(&mut p)
    }
}

impl Empty {
    fn parse_(p_: &mut xflags::rt::Parser) -> xflags::Result<Self> {
        #![allow(non_snake_case, unused_mut)]

        let mut state_ = 0u8;
        if let Some(arg_) = p_.pop_flag() {
            match arg_ {
                Ok(flag_) => match (state_, flag_.as_str()) {
                    (0, "--help" | "-h") => return Err(p_.help(Self::HELP_)),
                    _ => return Err(p_.unexpected_flag(&flag_).chain("\n\n").chain(Self::HELP_)),
                },
                Err(arg_) => match (state_, arg_.to_str().unwrap_or("")) {
                    (0, "help") => return Err(p_.help(Self::HELP_)),
                    _ => return Err(p_.unexpected_arg(arg_).chain("\n\n").chain(Self::HELP_)),
                },
            }
        }
        Ok(Empty {})
    }
}
impl Empty {
    const HELP_: &'static str = "Usage: empty [-h]
Options:
  -h, --help           Prints help

Commands:
  help                 Print this message or the help of the given subcommand(s)";
}
