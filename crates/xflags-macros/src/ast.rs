#[derive(Debug)]
pub(crate) struct XFlags {
    pub(crate) src: Option<String>,
    pub(crate) cmd: Cmd,
}

impl XFlags {
    pub fn is_anon(&self) -> bool {
        self.cmd.name.is_empty()
    }
}

#[derive(Debug)]
pub(crate) struct Cmd {
    pub(crate) name: String,
    pub(crate) doc: Option<String>,
    pub(crate) args: Vec<Arg>,
    pub(crate) flags: Vec<Flag>,
    pub(crate) subcommands: Vec<Cmd>,
    pub(crate) default: bool,
    pub(crate) idx: u8,
}

#[derive(Debug)]
pub(crate) struct Arg {
    pub(crate) arity: Arity,
    pub(crate) doc: Option<String>,
    pub(crate) val: Val,
}

#[derive(Debug)]
pub(crate) struct Flag {
    pub(crate) arity: Arity,
    pub(crate) name: String,
    pub(crate) short: Option<String>,
    pub(crate) doc: Option<String>,
    pub(crate) val: Option<Val>,
}

impl Flag {
    pub(crate) fn is_help(&self) -> bool {
        self.name == "help"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Arity {
    Optional,
    Required,
    Repeated,
}

#[derive(Debug)]
pub(crate) struct Val {
    pub(crate) name: String,
    pub(crate) ty: Ty,
}

#[derive(Debug)]
pub(crate) enum Ty {
    PathBuf,
    OsString,
    FromStr(String),
}
