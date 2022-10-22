use std::{ffi::OsString, fmt, str::FromStr};

use crate::{Error, Result};

macro_rules! format_err {
    ($($tt:tt)*) => {
        Error { msg: format!($($tt)*), help: false }
    };
}

macro_rules! bail {
    ($($tt:tt)*) => {
        return Err(format_err!($($tt)*))
    };
}

pub struct Parser {
    after_double_dash: bool,
    rargs: Vec<OsString>,
}

impl Parser {
    pub fn new(mut args: Vec<OsString>) -> Self {
        args.reverse();
        Self { after_double_dash: false, rargs: args }
    }

    pub fn new_from_env() -> Self {
        let args = std::env::args_os().collect::<Vec<_>>();
        let mut res = Parser::new(args);
        let _progn = res.next();
        res
    }

    pub fn pop_flag(&mut self) -> Option<Result<String, OsString>> {
        if self.after_double_dash {
            self.next().map(Err)
        } else {
            let arg = self.next()?;
            let arg_str = arg.to_str().unwrap_or_default();
            if arg_str.starts_with('-') {
                if arg_str == "--" {
                    self.after_double_dash = true;
                    return self.next().map(Err);
                }
                Some(arg.into_string())
            } else {
                Some(Err(arg))
            }
        }
    }

    pub fn push_back(&mut self, arg: Result<String, OsString>) {
        let arg = match arg {
            Ok(it) => it.into(),
            Err(it) => it,
        };
        self.rargs.push(arg)
    }

    fn next(&mut self) -> Option<OsString> {
        self.rargs.pop()
    }

    pub fn next_value(&mut self, flag: &str) -> Result<OsString> {
        self.next().ok_or_else(|| format_err!("expected a value for `{flag}`"))
    }

    pub fn next_value_from_str<T: FromStr>(&mut self, flag: &str) -> Result<T>
    where
        T::Err: fmt::Display,
    {
        let value = self.next_value(flag)?;
        self.value_from_str(flag, value)
    }

    pub fn value_from_str<T: FromStr>(&mut self, flag: &str, value: OsString) -> Result<T>
    where
        T::Err: fmt::Display,
    {
        match value.into_string() {
            Ok(str) => str.parse::<T>().map_err(|err| format_err!("can't parse `{flag}`, {err}")),
            Err(it) => {
                bail!("can't parse `{flag}`, invalid utf8: {it:?}")
            }
        }
    }

    pub fn unexpected_flag(&self, flag: &str) -> Error {
        format_err!("unexpected flag: `{flag}`")
    }

    pub fn unexpected_arg(&self, arg: OsString) -> Error {
        format_err!("unexpected argument: {arg:?}")
    }

    pub fn subcommand_required(&self) -> Error {
        format_err!("subcommand is required")
    }

    pub fn help(&self, help: &'static str) -> Error {
        Error { msg: help.to_string(), help: true }
    }

    pub fn optional<T>(&self, flag: &str, mut vals: Vec<T>) -> Result<Option<T>> {
        if vals.len() > 1 {
            bail!("flag specified more than once: `{flag}`")
        }
        Ok(vals.pop())
    }

    pub fn required<T>(&self, flag: &str, mut vals: Vec<T>) -> Result<T> {
        if vals.len() > 1 {
            bail!("flag specified more than once: `{flag}`")
        }
        vals.pop().ok_or_else(|| format_err!("flag is required: `{flag}`"))
    }
}
