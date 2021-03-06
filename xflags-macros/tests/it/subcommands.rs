#![allow(unused)]
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug)]
pub struct RustAnalyzer {
    pub verbose: u32,
    pub subcommand: RustAnalyzerCmd,
}

#[derive(Debug)]
pub enum RustAnalyzerCmd {
    Server(Server),
    AnalysisStats(AnalysisStats),
}

#[derive(Debug)]
pub struct Server {
    pub dir: Option<PathBuf>,
    pub subcommand: ServerCmd,
}

#[derive(Debug)]
pub enum ServerCmd {
    Launch(Launch),
    Watch(Watch),
}

#[derive(Debug)]
pub struct Launch {
    pub log: bool,
}

#[derive(Debug)]
pub struct Watch;

#[derive(Debug)]
pub struct AnalysisStats {
    pub path: PathBuf,

    pub parallel: bool,
}

impl RustAnalyzer {
    pub const HELP: &'static str = Self::HELP_;

    pub fn from_env() -> xflags::Result<Self> {
        Self::from_env_()
    }

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
        let mut verbose = Vec::new();

        let mut sub_ = None;
        while let Some(arg_) = p_.pop_flag() {
            match arg_ {
                Ok(flag_) => match flag_.as_str() {
                    "--verbose" | "-v" => verbose.push(()),
                    _ => return Err(p_.unexpected_flag(&flag_)),
                },
                Err(arg_) => {
                    match arg_.to_str().unwrap_or("") {
                        "server" => {
                            sub_ = Some(RustAnalyzerCmd::Server(Server::parse_(p_)?));
                            break;
                        }
                        "analysis-stats" => {
                            sub_ = Some(RustAnalyzerCmd::AnalysisStats(AnalysisStats::parse_(p_)?));
                            break;
                        }
                        _ => (),
                    }
                    return Err(p_.unexpected_arg(arg_));
                }
            }
        }
        Ok(Self { verbose: verbose.len() as u32, subcommand: p_.subcommand(sub_)? })
    }
}

impl Server {
    fn parse_(p_: &mut xflags::rt::Parser) -> xflags::Result<Self> {
        let mut dir = Vec::new();

        let mut sub_ = None;
        while let Some(arg_) = p_.pop_flag() {
            match arg_ {
                Ok(flag_) => match flag_.as_str() {
                    "--dir" => dir.push(p_.next_value(&flag_)?.into()),
                    _ => {
                        p_.push_back(Ok(flag_));
                        break;
                    }
                },
                Err(arg_) => {
                    match arg_.to_str().unwrap_or("") {
                        "watch" => {
                            sub_ = Some(ServerCmd::Watch(Watch::parse_(p_)?));
                            break;
                        }
                        _ => (),
                    }
                    p_.push_back(Err(arg_));
                    break;
                }
            }
        }
        if sub_.is_none() {
            sub_ = Some(ServerCmd::Launch(Launch::parse_(p_)?));
        }
        Ok(Self { dir: p_.optional("--dir", dir)?, subcommand: p_.subcommand(sub_)? })
    }
}

impl Launch {
    fn parse_(p_: &mut xflags::rt::Parser) -> xflags::Result<Self> {
        let mut log = Vec::new();

        while let Some(arg_) = p_.pop_flag() {
            match arg_ {
                Ok(flag_) => match flag_.as_str() {
                    "--log" => log.push(()),
                    _ => return Err(p_.unexpected_flag(&flag_)),
                },
                Err(arg_) => {
                    return Err(p_.unexpected_arg(arg_));
                }
            }
        }
        Ok(Self { log: p_.optional("--log", log)?.is_some() })
    }
}

impl Watch {
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

impl AnalysisStats {
    fn parse_(p_: &mut xflags::rt::Parser) -> xflags::Result<Self> {
        let mut parallel = Vec::new();

        let mut path = (false, Vec::new());

        while let Some(arg_) = p_.pop_flag() {
            match arg_ {
                Ok(flag_) => match flag_.as_str() {
                    "--parallel" => parallel.push(()),
                    _ => return Err(p_.unexpected_flag(&flag_)),
                },
                Err(arg_) => {
                    if let (done_ @ false, buf_) = &mut path {
                        buf_.push(arg_.into());
                        *done_ = true;
                        continue;
                    }
                    return Err(p_.unexpected_arg(arg_));
                }
            }
        }
        Ok(Self {
            path: p_.required("path", path.1)?,

            parallel: p_.optional("--parallel", parallel)?.is_some(),
        })
    }
}
impl RustAnalyzer {
    const HELP_: &'static str = "rust-analyzer

OPTIONS:
    -v, --verbose

SUBCOMANDS:

rust-analyzer server

  OPTIONS:
    --dir <path>

    --log


rust-analyzer server watch


rust-analyzer analysis-stats

  ARGS:
    <path>

  OPTIONS:
    --parallel
";
}
