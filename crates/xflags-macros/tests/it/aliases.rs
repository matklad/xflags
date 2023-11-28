#![allow(unused)]
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug)]
pub struct AliasCmd {
    pub subcommand: AliasCmdCmd,
}

#[derive(Debug)]
pub enum AliasCmdCmd {
    Sub(Sub),
    This(This),
}

#[derive(Debug)]
pub struct Sub {
    pub count: Option<usize>,
}

#[derive(Debug)]
pub struct This;

impl AliasCmd {
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

impl AliasCmd {
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

impl AliasCmd {
    fn parse_(p_: &mut xflags::rt::Parser) -> xflags::Result<Self> {
        #![allow(non_snake_case)]
        let mut sub__count = Vec::new();

        let mut state_ = 0u8;
        while let Some(arg_) = p_.pop_flag() {
            match arg_ {
                Ok(flag_) => match (state_, flag_.as_str()) {
                    (0 | 1 | 2, "--help" | "-h") => return Err(p_.help(Self::HELP_)),
                    (1, "--count" | "-c") => {
                        sub__count.push(p_.next_value_from_str::<usize>(&flag_)?)
                    }
                    _ => return Err(p_.unexpected_flag(&flag_)),
                },
                Err(arg_) => match (state_, arg_.to_str().unwrap_or("")) {
                    (0, "sub" | "s") => state_ = 1,
                    (0, "this" | "one" | "has" | "a" | "lot" | "of" | "aliases") => state_ = 2,
                    (0, _) => {
                        return Err(p_.unexpected_arg(arg_));
                    }
                    _ => return Err(p_.unexpected_arg(arg_)),
                },
            }
        }
        Ok(AliasCmd {
            subcommand: match state_ {
                1 => AliasCmdCmd::Sub(Sub { count: p_.optional("--count", sub__count)? }),
                2 => AliasCmdCmd::This(This {}),
                _ => return Err(p_.subcommand_required()),
            },
        })
    }
}
impl AliasCmd {
    const HELP_: &'static str = "\
alias-cmd
  commands with different aliases

OPTIONS:
    -h, --help
      Prints help information.

SUBCOMMANDS:

alias-cmd sub | s
  And even an aliased subcommand!

  OPTIONS:
    -c, --count <count>
      Little sanity check to see if this still works as intended


alias-cmd this | one | has | a | lot | of | aliases
";
}
