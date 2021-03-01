use std::{ffi::OsString, fmt, str::FromStr};

use crate::{Error, Result};

macro_rules! format_err {
    ($($tt:tt)*) => {
        Error { msg: format!($($tt)*) }
    };
}

macro_rules! bail {
    ($($tt:tt)*) => {
        return Err(format_err!($($tt)*))
    };
}

pub struct Parser {
    rargs: Vec<OsString>,
}

impl Parser {
    pub fn new(mut args: Vec<OsString>) -> Self {
        args.reverse();
        Self { rargs: args }
    }

    pub fn new_from_env() -> Self {
        let mut args = std::env::args_os().collect::<Vec<_>>();
        args.reverse();
        args.pop();
        Self { rargs: args }
    }

    pub fn is_empty(&self) -> bool {
        self.rargs.is_empty()
    }

    pub fn peek_flag(&self) -> Option<&str> {
        self.rargs.last().and_then(|it| it.to_str()).filter(|it| it.starts_with('-'))
    }
    pub fn pop_flag(&mut self) -> Option<Result<String, OsString>> {
        if self.peek_flag().is_some() {
            self.next().map(|it| it.into_string())
        } else {
            self.next().map(Err)
        }
    }
    pub fn push_back(&mut self, arg: Result<String, OsString>) {
        let arg = match arg {
            Ok(it) => it.into(),
            Err(it) => it,
        };
        self.rargs.push(arg)
    }

    pub fn next(&mut self) -> Option<OsString> {
        self.rargs.pop()
    }

    pub fn next_value(&mut self, flag: &str) -> Result<OsString> {
        if self.peek_flag().is_some() {
            bail!("expected a value for `{}`", flag)
        }
        self.next().ok_or_else(|| format_err!("expected a value for `{}`", flag))
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
            Ok(str) => {
                str.parse::<T>().map_err(|err| format_err!("can't parse `{}`, {}", flag, err))
            }
            Err(it) => {
                bail!("can't parse `{}`, invalid utf8: {:?}", flag, it)
            }
        }
    }

    pub fn unexpected_flag(&self, flag: &str) -> Error {
        format_err!("unexpected flag: `{}`", flag)
    }

    pub fn unexpected_arg(&self, arg: OsString) -> Error {
        format_err!("unexpected argument: {:?}", arg)
    }

    pub fn optional<T>(&self, flag: &str, mut vals: Vec<T>) -> Result<Option<T>> {
        if vals.len() > 1 {
            bail!("flag specified more than once: `{}`", flag)
        }
        Ok(vals.pop())
    }

    pub fn required<T>(&self, flag: &str, mut vals: Vec<T>) -> Result<T> {
        if vals.len() > 1 {
            bail!("flag specified more than once: `{}`", flag)
        }
        vals.pop().ok_or_else(|| format_err!("flag is required: `{}`", flag))
    }

    pub fn subcommand<T>(&self, cmd: Option<T>) -> Result<T> {
        cmd.ok_or_else(|| format_err!("subcommand is required"))
    }
}
