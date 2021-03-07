#![allow(unused)]
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug)]
pub struct Helpful {
    pub src: Option<PathBut>,

    pub switch: (),
    pub subcommand: HelpfulCmd,
}

#[derive(Debug)]
pub enum HelpfulCmd {
    Sub(Sub),
}

#[derive(Debug)]
pub struct Sub;

impl Helpful {
    pub const HELP: &'static str = Self::HELP_;

    pub fn from_env() -> xflags::Result<Self> {
        Self::from_env_()
    }

    pub fn from_vec(args: Vec<std::ffi::OsString>) -> xflags::Result<Self> {
        Self::from_vec_(args)
    }
}

impl Helpful {
    fn from_env_() -> xflags::Result<Self> {
        let mut p = xflags::rt::Parser::new_from_env();
        Self::parse_(&mut p)
    }
    fn from_vec_(args: Vec<std::ffi::OsString>) -> xflags::Result<Self> {
        let mut p = xflags::rt::Parser::new(args);
        Self::parse_(&mut p)
    }
}

impl Helpful {
    fn parse_(p_: &mut xflags::rt::Parser) -> xflags::Result<Self> {
        let mut switch = Vec::new();

        let mut src = (false, Vec::new());

        let mut sub_ = None;
        while let Some(arg_) = p_.pop_flag() {
            match arg_ {
                Ok(flag_) => match flag_.as_str() {
                    "--switch" | "-s" => switch.push(()),
                    _ => return Err(p_.unexpected_flag(&flag_)),
                },
                Err(arg_) => {
                    match arg_.to_str().unwrap_or("") {
                        "sub" => {
                            sub_ = Some(HelpfulCmd::Sub(Sub::parse_(p_)?));
                            break;
                        }
                        _ => (),
                    }
                    if let (done_ @ false, buf_) = &mut src {
                        buf_.push(p_.value_from_str::<PathBut>("src", arg_)?);
                        *done_ = true;
                        continue;
                    }
                    return Err(p_.unexpected_arg(arg_));
                }
            }
        }
        Ok(Self {
            src: p_.optional("src", src.1)?,

            switch: p_.required("--switch", switch)?,
            subcommand: p_.subcommand(sub_)?,
        })
    }
}

impl Sub {
    fn parse_(p_: &mut xflags::rt::Parser) -> xflags::Result<Self> {
        while let Some(arg_) = p_.pop_flag() {
            match arg_ {
                Ok(flag_) => match flag_.as_str() {
                    _ => return Err(p_.unexpected_flag(&flag_)),
                },
                Err(arg_) => {
                    return Err(p_.unexpected_arg(arg_));
                }
            }
        }
        Ok(Self {})
    }
}
impl Helpful {
    const HELP_: &'static str = "helpful
  Does stuff

ARGS:
    [src]
      With an arg.

OPTIONS:
    -s, --switch
      And a switch.

SUBCOMMANDS:

helpful sub
  And even a subcommand!
";
}
