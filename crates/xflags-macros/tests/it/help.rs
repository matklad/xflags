#![allow(unused)]
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug)]
pub struct Helpful {
    pub src: Option<PathBuf>,
    pub extra: Option<String>,

    pub switch: (),
    pub subcommand: HelpfulCmd,
}

#[derive(Debug)]
pub enum HelpfulCmd {
    Sub(Sub),
}

#[derive(Debug)]
pub struct Sub {
    pub flag: bool,
}

impl Helpful {
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

impl Helpful {
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

impl Helpful {
    fn parse_(p_: &mut xflags::rt::Parser) -> xflags::Result<Self> {
        #![allow(non_snake_case)]
        let mut switch = Vec::new();
        let mut src = (false, Vec::new());
        let mut extra = (false, Vec::new());
        let mut sub__flag = Vec::new();

        let mut state_ = 0u8;
        while let Some(arg_) = p_.pop_flag() {
            match arg_ {
                Ok(flag_) => match (state_, flag_.as_str()) {
                    (0 | 1, "--switch" | "-s") => switch.push(()),
                    (0 | 1, "--help" | "-h") => return Err(p_.help(Self::HELP_)),
                    (1, "--flag" | "-f") => sub__flag.push(()),
                    _ => return Err(p_.unexpected_flag(&flag_)),
                },
                Err(arg_) => match (state_, arg_.to_str().unwrap_or("")) {
                    (0, "sub") => state_ = 1,
                    (0, _) => {
                        if let (done_ @ false, buf_) = &mut src {
                            buf_.push(arg_.into());
                            *done_ = true;
                            continue;
                        }
                        if let (done_ @ false, buf_) = &mut extra {
                            buf_.push(p_.value_from_str::<String>("extra", arg_)?);
                            *done_ = true;
                            continue;
                        }
                        return Err(p_.unexpected_arg(arg_));
                    }
                    _ => return Err(p_.unexpected_arg(arg_)),
                },
            }
        }
        Ok(Helpful {
            switch: p_.required("--switch", switch)?,
            src: p_.optional("src", src.1)?,
            extra: p_.optional("extra", extra.1)?,
            subcommand: match state_ {
                1 => HelpfulCmd::Sub(Sub { flag: p_.optional("--flag", sub__flag)?.is_some() }),
                _ => return Err(p_.subcommand_required()),
            },
        })
    }
}
impl Helpful {
    const HELP_: &'static str = "\
helpful
  Does stuff

  Helpful stuff.

ARGS:
    [src]
      With an arg.

    [extra]
      Another arg.

      This time, we provide some extra info about the
      arg. Maybe some caveats, or what kinds of
      values are accepted.

OPTIONS:
    -s, --switch
      And a switch.

    -h, --help
      Prints help information.

SUBCOMMANDS:

helpful sub
  And even a subcommand!

  OPTIONS:
    -f, --flag
      With an optional flag. This has a really long
      description which spans multiple lines.
";
}
