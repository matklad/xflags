#![allow(unused)]
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug)]
pub struct RustAnalyzer {
    pub workspace: PathBuf,
    pub jobs: Option<u32>,

    pub log_file: Option<PathBuf>,
    pub verbose: u32,
    pub number: u32,
    pub data: Vec<OsString>,
    pub emoji: bool,
}

impl RustAnalyzer {
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

impl RustAnalyzer {
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

impl RustAnalyzer {
    fn parse_(p_: &mut xflags::rt::Parser) -> xflags::Result<Self> {
        #![allow(non_snake_case)]
        let mut log_file = Vec::new();
        let mut verbose = Vec::new();
        let mut number = Vec::new();
        let mut data = Vec::new();
        let mut emoji = Vec::new();
        let mut workspace = (false, Vec::new());
        let mut jobs = (false, Vec::new());

        let mut state_ = 0u8;
        while let Some(arg_) = p_.pop_flag() {
            match arg_ {
                Ok(flag_) => match (state_, flag_.as_str()) {
                    (0, "--log-file") => log_file.push(p_.next_value(&flag_)?.into()),
                    (0, "--verbose" | "-v") => verbose.push(()),
                    (0, "--number" | "-n") => number.push(p_.next_value_from_str::<u32>(&flag_)?),
                    (0, "--data") => data.push(p_.next_value(&flag_)?.into()),
                    (0, "--emoji") => emoji.push(()),
                    (0, "--help" | "-h") => return Err(p_.help(Self::HELP_)),
                    _ => return Err(p_.unexpected_flag(&flag_)),
                },
                Err(arg_) => match (state_, arg_.to_str().unwrap_or("")) {
                    (0, _) => {
                        if let (done_ @ false, buf_) = &mut workspace {
                            buf_.push(arg_.into());
                            *done_ = true;
                            continue;
                        }
                        if let (done_ @ false, buf_) = &mut jobs {
                            buf_.push(p_.value_from_str::<u32>("jobs", arg_)?);
                            *done_ = true;
                            continue;
                        }
                        return Err(p_.unexpected_arg(arg_));
                    }
                    _ => return Err(p_.unexpected_arg(arg_)),
                },
            }
        }
        Ok(RustAnalyzer {
            log_file: p_.optional("--log-file", log_file)?,
            verbose: verbose.len() as u32,
            number: p_.required("--number", number)?,
            data: data,
            emoji: p_.optional("--emoji", emoji)?.is_some(),
            workspace: p_.required("workspace", workspace.1)?,
            jobs: p_.optional("jobs", jobs.1)?,
        })
    }
}
impl RustAnalyzer {
    const HELP_: &'static str = "\
rust-analyzer
  LSP server for rust.

ARGS:
    <workspace>

    [jobs]
      Number of concurrent jobs.

OPTIONS:
    --log-file <path>
      Path to log file. By default, logs go to stderr.

    -v, --verbose

    -n, --number <n>

    --data <value>

    --emoji

    -h, --help
      Prints help information.
";
}
