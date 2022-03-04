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
    pub malloc: Option<Malloc>,
}

#[derive(Debug)]
pub enum Malloc {
    Jemalloc,
    Mimalloc,
    Sys,
}

impl RustAnalyzer {
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

impl RustAnalyzer {
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
        let mut log_file = Vec::new();
        let mut verbose = Vec::new();
        let mut number = Vec::new();
        let mut data = Vec::new();
        let mut emoji = Vec::new();
        let mut malloc = Vec::new();

        let mut workspace = (false, Vec::new());
        let mut jobs = (false, Vec::new());

        while let Some(arg_) = p_.pop_flag() {
            match arg_ {
                Ok(flag_) => match flag_.as_str() {
                    "--log-file" => log_file.push(p_.next_value(&flag_)?.into()),
                    "--verbose" | "-v" => verbose.push(()),
                    "--number" | "-n" => number.push(p_.next_value_from_str::<u32>(&flag_)?),
                    "--data" => data.push(p_.next_value(&flag_)?.into()),
                    "--emoji" => emoji.push(()),
                    "--malloc" => malloc.push(p_.next_value_from_str::<Malloc>(&flag_)?),
                    _ => return Err(p_.unexpected_flag(&flag_)),
                },
                Err(arg_) => {
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
            }
        }
        Ok(Self {
            workspace: p_.required("workspace", workspace.1)?,
            jobs: p_.optional("jobs", jobs.1)?,

            log_file: p_.optional("--log-file", log_file)?,
            verbose: verbose.len() as u32,
            number: p_.required("--number", number)?,
            data: data,
            emoji: p_.optional("--emoji", emoji)?.is_some(),
            malloc: p_.optional("--malloc", malloc)?,
        })
    }
}

impl core::str::FromStr for Malloc {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "jemalloc" => Ok(Self::Jemalloc),
            "mimalloc" => Ok(Self::Mimalloc),
            "sys" => Ok(Self::Sys),
            s => Err(format!("unknown value for `malloc`: {:?}", s)),
        }
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

    --malloc <jemalloc | mimalloc | sys>
";
}
