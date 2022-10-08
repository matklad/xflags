use std::{fmt, mem};

#[cfg(not(test))]
use proc_macro::{Delimiter, TokenStream, TokenTree};
#[cfg(test)]
use proc_macro2::{Delimiter, TokenStream, TokenTree};

use crate::ast;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub(crate) struct Error {
    msg: String,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.msg, f)
    }
}

pub(crate) fn xflags(ts: TokenStream) -> Result<ast::XFlags> {
    let p = &mut Parser::new(ts);
    let src = if p.eat_keyword("src") { Some(p.expect_string()?) } else { None };
    let doc = opt_doc(p)?;
    let mut cmd = cmd(p)?;
    cmd.doc = doc;
    add_help(&mut cmd);
    let res = ast::XFlags { src, cmd };
    Ok(res)
}

pub(crate) fn parse_or_exit(ts: TokenStream) -> Result<ast::XFlags> {
    let p = &mut Parser::new(ts);
    let mut cmd = anon_cmd(p)?;
    assert!(cmd.subcommands.is_empty());
    add_help(&mut cmd);
    let res = ast::XFlags { src: None, cmd };
    Ok(res)
}

fn add_help(cmd: &mut ast::Cmd) {
    let help = ast::Flag {
        arity: ast::Arity::Optional,
        name: "help".to_string(),
        short: Some("h".to_string()),
        doc: Some("Prints help information.".to_string()),
        val: None,
    };
    cmd.flags.push(help);
}

macro_rules! format_err {
    ($($tt:tt)*) => {
        Error { msg: format!($($tt)*) }
        // panic!($($tt)*)
    };
}

macro_rules! bail {
    ($($tt:tt)*) => {
        return Err(format_err!($($tt)*))
    };
}

fn anon_cmd(p: &mut Parser) -> Result<ast::Cmd> {
    cmd_impl(p, true)
}

fn cmd(p: &mut Parser) -> Result<ast::Cmd> {
    cmd_impl(p, false)
}

fn cmd_impl(p: &mut Parser, anon: bool) -> Result<ast::Cmd> {
    let name = if anon {
        String::new()
    } else {
        p.expect_keyword("cmd")?;
        cmd_name(p)?
    };

    let idx = p.idx;
    p.idx += 1;

    let mut res = ast::Cmd {
        name,
        doc: None,
        args: Vec::new(),
        flags: Vec::new(),
        subcommands: Vec::new(),
        default: false,
        idx,
    };

    if !anon {
        p.enter_delim(Delimiter::Brace)?;
    }
    while !p.end() {
        let doc = opt_doc(p)?;
        let default = !anon && p.eat_keyword("default");
        if !anon && (default || p.at_keyword("cmd")) {
            let mut cmd = cmd(p)?;
            cmd.doc = doc;
            res.subcommands.push(cmd);
            if default {
                if res.default {
                    bail!("only one subcommand can be default")
                }
                res.default = true;
                res.subcommands.rotate_right(1);
            }
        } else {
            let arity = arity(p)?;
            let is_val = p.lookahead_punct(':', 1);
            let name = p.expect_name()?;
            if name.starts_with('-') {
                let mut flag = flag(p, name)?;
                flag.doc = doc;
                flag.arity = arity;
                res.flags.push(flag)
            } else if is_val {
                p.expect_punct(':')?;
                let ty = ty(p)?;
                let val = ast::Val { name, ty };
                let arg = ast::Arg { arity, doc, val };
                res.args.push(arg);
            } else {
                bail!("expected `--flag` or `arg: Type`")
            }
        }
    }
    if !anon {
        p.exit_delim()?;
    }
    Ok(res)
}

fn flag(p: &mut Parser, name: String) -> Result<ast::Flag> {
    let short;
    let long;
    if name.starts_with("--") {
        short = None;
        long = name;
    } else {
        short = Some(name);
        if !p.eat_punct(',') {
            bail!("long option is required for `{}`", short.unwrap());
        }
        long = flag_name(p)?;
        if !long.starts_with("--") {
            bail!("long name must begin with `--`: `{long}`");
        }
    }

    if long == "--help" {
        bail!("`--help` flag is generated automatically")
    }

    let val = opt_val(p)?;
    Ok(ast::Flag {
        arity: ast::Arity::Required,
        name: long[2..].to_string(),
        short: short.map(|it| it[1..].to_string()),
        doc: None,
        val,
    })
}

fn opt_val(p: &mut Parser) -> Result<Option<ast::Val>, Error> {
    if !p.lookahead_punct(':', 1) {
        return Ok(None);
    }

    let name = p.expect_name()?;
    p.expect_punct(':')?;
    let ty = ty(p)?;
    let res = ast::Val { name, ty };
    Ok(Some(res))
}

fn arity(p: &mut Parser) -> Result<ast::Arity> {
    if p.eat_keyword("optional") {
        return Ok(ast::Arity::Optional);
    }
    if p.eat_keyword("required") {
        return Ok(ast::Arity::Required);
    }
    if p.eat_keyword("repeated") {
        return Ok(ast::Arity::Repeated);
    }
    if let Some(name) = p.eat_name() {
        bail!("expected one of `optional`, `required`, `repeated`, got `{name}`")
    }
    bail!("expected one of `optional`, `required`, `repeated`, got {:?}", p.ts.pop())
}

fn ty(p: &mut Parser) -> Result<ast::Ty> {
    let name = p.expect_name()?;
    let res = match name.as_str() {
        "PathBuf" => ast::Ty::PathBuf,
        "OsString" => ast::Ty::OsString,
        _ => ast::Ty::FromStr(name),
    };
    Ok(res)
}

fn opt_single_doc(p: &mut Parser) -> Result<Option<String>> {
    if !p.eat_punct('#') {
        return Ok(None);
    }
    p.enter_delim(Delimiter::Bracket)?;
    p.expect_keyword("doc")?;
    p.expect_punct('=')?;
    let mut res = p.expect_string()?;
    if let Some(suf) = res.strip_prefix(' ') {
        res = suf.to_string();
    }
    p.exit_delim()?;
    Ok(Some(res))
}

fn opt_doc(p: &mut Parser) -> Result<Option<String>> {
    let lines =
        core::iter::from_fn(|| opt_single_doc(p).transpose()).collect::<Result<Vec<String>>>()?;
    let lines = lines.join("\n");

    if lines.is_empty() {
        Ok(None)
    } else {
        Ok(Some(lines))
    }
}

fn cmd_name(p: &mut Parser) -> Result<String> {
    let name = p.expect_name()?;
    if name.starts_with('-') {
        bail!("command name can't begin with `-`: `{name}`");
    }
    Ok(name)
}

fn flag_name(p: &mut Parser) -> Result<String> {
    let name = p.expect_name()?;
    if !name.starts_with('-') {
        bail!("flag name should begin with `-`: `{name}`");
    }
    Ok(name)
}

struct Parser {
    stack: Vec<Vec<TokenTree>>,
    ts: Vec<TokenTree>,
    idx: u8,
}

impl Parser {
    fn new(ts: TokenStream) -> Self {
        let mut ts = ts.into_iter().collect::<Vec<_>>();
        ts.reverse();
        Self { stack: Vec::new(), ts, idx: 0 }
    }

    fn enter_delim(&mut self, delimiter: Delimiter) -> Result<()> {
        match self.ts.pop() {
            Some(TokenTree::Group(g)) if g.delimiter() == delimiter => {
                let mut ts = g.stream().into_iter().collect::<Vec<_>>();
                ts.reverse();
                let ts = mem::replace(&mut self.ts, ts);
                self.stack.push(ts);
            }
            _ => bail!("expected `{{`"),
        }
        Ok(())
    }
    fn exit_delim(&mut self) -> Result<()> {
        if !self.end() {
            bail!("expected `}}`")
        }
        self.ts = self.stack.pop().unwrap();
        Ok(())
    }
    fn end(&mut self) -> bool {
        self.ts.last().is_none()
    }

    fn expect_keyword(&mut self, kw: &str) -> Result<()> {
        if !self.eat_keyword(kw) {
            bail!("expected `{kw}`")
        }
        Ok(())
    }
    fn eat_keyword(&mut self, kw: &str) -> bool {
        if self.at_keyword(kw) {
            self.ts.pop().unwrap();
            true
        } else {
            false
        }
    }
    fn at_keyword(&mut self, kw: &str) -> bool {
        match self.ts.last() {
            Some(TokenTree::Ident(ident)) => &ident.to_string() == kw,
            _ => false,
        }
    }

    fn expect_name(&mut self) -> Result<String> {
        self.eat_name().ok_or_else(|| {
            let next = self.ts.pop().map(|it| it.to_string()).unwrap_or_default();
            format_err!("expected a name, got: `{next}`")
        })
    }
    fn eat_name(&mut self) -> Option<String> {
        let mut buf = String::new();
        let mut prev_ident = false;
        loop {
            match self.ts.last() {
                Some(TokenTree::Punct(p)) if p.as_char() == '-' => {
                    prev_ident = false;
                    buf.push('-');
                }
                Some(TokenTree::Ident(ident)) if !prev_ident => {
                    prev_ident = true;
                    buf.push_str(&ident.to_string());
                }
                _ => break,
            }
            self.ts.pop();
        }
        if buf.is_empty() {
            None
        } else {
            Some(buf)
        }
    }

    fn _expect_ident(&mut self) -> Result<String> {
        match self.ts.pop() {
            Some(TokenTree::Ident(ident)) => Ok(ident.to_string()),
            _ => bail!("expected ident"),
        }
    }

    fn expect_punct(&mut self, punct: char) -> Result<()> {
        if !self.eat_punct(punct) {
            bail!("expected `{punct}`")
        }
        Ok(())
    }
    fn eat_punct(&mut self, punct: char) -> bool {
        match self.ts.last() {
            Some(TokenTree::Punct(p)) if p.as_char() == punct => {
                self.ts.pop();
                true
            }
            _ => false,
        }
    }
    fn lookahead_punct(&mut self, punct: char, n: usize) -> bool {
        match self.ts.iter().rev().nth(n) {
            Some(TokenTree::Punct(p)) => p.as_char() == punct,
            _ => false,
        }
    }

    fn expect_string(&mut self) -> Result<String> {
        match self.ts.pop() {
            Some(TokenTree::Literal(lit)) if lit.to_string().starts_with('"') => {
                let res = str_lit_value(lit.to_string());
                Ok(res)
            }
            _ => bail!("expected a string"),
        }
    }
}

/// "Parser" a string literal into the corresponding value.
///
/// Really needs support in the proc_macro library:
/// <https://internals.rust-lang.org/t/getting-value-out-of-proc-macro-literal/14140>
fn str_lit_value(lit: String) -> String {
    lit.trim_matches('"').replace("\\'", "'")
}
