#[allow(unused)]
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
        #![allow(non_snake_case, unused_mut)]
        let mut verbose = Vec::new();
        let mut server__dir = Vec::new();
        let mut server__launch__log = Vec::new();
        let mut analysis_stats__parallel = Vec::new();
        let mut analysis_stats__path = (false, Vec::new());

        let mut state_ = 0u8;
        while let Some(arg_) = p_.pop_flag() {
            match arg_ {
                Ok(flag_) => match (state_, flag_.as_str()) {
                    (0, "--help" | "-h") => return Err(p_.help(Self::HELP_)),
                    (0 | 1 | 2 | 3 | 4, "--verbose" | "-v") => verbose.push(()),
                    (1, "--help" | "-h") => return Err(p_.help(Self::HELP_SERVER__)),
                    (1 | 2 | 3, "--dir") => server__dir.push(p_.next_value(&flag_)?.into()),
                    (1, _) => {
                        p_.push_back(Ok(flag_));
                        state_ = 2;
                    }
                    (2, "--help" | "-h") => return Err(p_.help(Self::HELP_SERVER__LAUNCH__)),
                    (2, "--log") => server__launch__log.push(()),
                    (3, "--help" | "-h") => return Err(p_.help(Self::HELP_SERVER__WATCH__)),
                    (4, "--help" | "-h") => return Err(p_.help(Self::HELP_ANALYSIS_STATS__)),
                    (4, "--parallel") => analysis_stats__parallel.push(()),
                    _ => return Err(p_.unexpected_flag(&flag_)),
                },
                Err(arg_) => match (state_, arg_.to_str().unwrap_or("")) {
                    (0, "server") => state_ = 1,
                    (0, "analysis-stats") => state_ = 4,
                    (0, "help") => return Err(p_.help(Self::HELP_)),
                    (0, _) => {
                        return Err(p_.unexpected_arg(arg_));
                    }
                    (1, "watch") => state_ = 3,
                    (1, "help") => return Err(p_.help(Self::HELP_SERVER__)),
                    (1, _) => {
                        p_.push_back(Err(arg_));
                        state_ = 2;
                    }
                    (2, "help") => return Err(p_.help(Self::HELP_SERVER__LAUNCH__)),
                    (3, "help") => return Err(p_.help(Self::HELP_SERVER__WATCH__)),
                    (4, _) => {
                        if let (done_ @ false, buf_) = &mut analysis_stats__path {
                            buf_.push(arg_.into());
                            *done_ = true;
                            continue;
                        }
                        return Err(p_.unexpected_arg(arg_));
                    }
                    _ => return Err(p_.unexpected_arg(arg_)),
                },
            }
        }
        state_ = if state_ == 1 { 2 } else { state_ };
        Ok(RustAnalyzer {
            verbose: verbose.len() as u32,
            subcommand: match state_ {
                2 | 3 => RustAnalyzerCmd::Server(Server {
                    dir: p_.optional("--dir", server__dir)?,
                    subcommand: match state_ {
                        2 => ServerCmd::Launch(Launch {
                            log: p_.optional("--log", server__launch__log)?.is_some(),
                        }),
                        3 => ServerCmd::Watch(Watch {}),
                        _ => return Err(p_.subcommand_required()),
                    },
                }),
                4 => RustAnalyzerCmd::AnalysisStats(AnalysisStats {
                    parallel: p_.optional("--parallel", analysis_stats__parallel)?.is_some(),
                    path: p_.required("path", analysis_stats__path.1)?,
                }),
                _ => return Err(p_.subcommand_required()),
            },
        })
    }
}
impl RustAnalyzer {
    const HELP_SERVER__LAUNCH__: &'static str = "Usage: launch [--log]
Options:
  --log                

Commands:
  help                 Print this message or the help of the given subcommand(s)";
    const HELP_SERVER__WATCH__: &'static str = "Usage: watch
Commands:
  help                 Print this message or the help of the given subcommand(s)";
    const HELP_SERVER__: &'static str = "Usage: server [--dir <path>] [--log] <COMMAND>
Options:
  --dir <path>         
  --log                

Commands:
  launch               
  watch                
  help                 Print this message or the help of the given subcommand(s)";
    const HELP_ANALYSIS_STATS__: &'static str = "Usage: analysis-stats <path> [--parallel]
Arguments:
  <path>               

Options:
  --parallel           

Commands:
  help                 Print this message or the help of the given subcommand(s)";
    const HELP_: &'static str = "Usage: rust-analyzer [-v]... [-h] <COMMAND>
Options:
  -v, --verbose        
  -h, --help           Prints help

Commands:
  server               
  analysis-stats       
  help                 Print this message or the help of the given subcommand(s)";
}
