use crate::{ast, update};

use std::{fmt::Write, path::Path};

pub(crate) fn emit(xflags: &ast::XFlags) -> String {
    let mut buf = String::new();

    emit_cmd(&mut buf, &xflags.cmd);
    blank_line(&mut buf);
    emit_api(&mut buf, xflags);

    if std::env::var("UPDATE_XFLAGS").is_ok() {
        if let Some(src) = &xflags.src {
            update::in_place(&buf, Path::new(src.as_str()))
        } else {
            update::stdout(&buf);
        }
    }

    if xflags.src.is_some() {
        buf.clear()
    }

    blank_line(&mut buf);
    emit_impls(&mut buf, &xflags);
    emit_help(&mut buf, xflags);

    buf
}

macro_rules! w {
    ($($tt:tt)*) => {
        drop(write!($($tt)*))
    };
}

fn emit_cmd(buf: &mut String, cmd: &ast::Cmd) {
    w!(buf, "#[derive(Debug)]\n");
    w!(buf, "pub struct {}", cmd.ident());
    if cmd.args.is_empty() && cmd.flags.is_empty() && cmd.subcommands.is_empty() {
        w!(buf, ";\n");
        return;
    }
    w!(buf, " {{\n");

    for arg in &cmd.args {
        let ty = gen_arg_ty(arg.arity, &arg.val.ty);
        w!(buf, "    pub {}: {},\n", arg.val.ident(), ty);
    }

    if !cmd.args.is_empty() && !cmd.flags.is_empty() {
        blank_line(buf);
    }

    for flag in &cmd.flags {
        let ty = gen_flag_ty(flag.arity, flag.val.as_ref().map(|it| &it.ty));
        w!(buf, "    pub {}: {},\n", flag.ident(), ty);
    }

    if cmd.has_subcommands() {
        w!(buf, "    pub subcommand: {},\n", cmd.cmd_enum_ident());
    }
    w!(buf, "}}\n");

    if cmd.has_subcommands() {
        blank_line(buf);
        w!(buf, "#[derive(Debug)]\n");
        w!(buf, "pub enum {} {{\n", cmd.cmd_enum_ident());
        for sub in &cmd.subcommands {
            let name = camel(&sub.name);
            w!(buf, "    {}({}),\n", name, name);
        }
        w!(buf, "}}\n");

        for sub in &cmd.subcommands {
            blank_line(buf);
            emit_cmd(buf, sub);
        }
    }
}

fn gen_flag_ty(arity: ast::Arity, ty: Option<&ast::Ty>) -> String {
    match ty {
        None => match arity {
            ast::Arity::Optional => "bool".to_string(),
            ast::Arity::Required => "()".to_string(),
            ast::Arity::Repeated => "u32".to_string(),
        },
        Some(ty) => gen_arg_ty(arity, ty),
    }
}

fn gen_arg_ty(arity: ast::Arity, ty: &ast::Ty) -> String {
    let ty = match ty {
        ast::Ty::PathBuf => "PathBuf".into(),
        ast::Ty::OsString => "OsString".into(),
        ast::Ty::FromStr(it) => it.clone(),
    };
    match arity {
        ast::Arity::Optional => format!("Option<{}>", ty),
        ast::Arity::Required => ty,
        ast::Arity::Repeated => format!("Vec<{}>", ty),
    }
}

fn emit_api(buf: &mut String, xflags: &ast::XFlags) {
    w!(buf, "impl {} {{\n", camel(&xflags.cmd.name));

    w!(buf, "    pub const HELP: &'static str = Self::HELP_;\n");
    blank_line(buf);

    w!(buf, "    #[allow(dead_code)]\n");
    w!(buf, "    pub fn from_env() -> xflags::Result<Self> {{\n");
    w!(buf, "        Self::from_env_()\n");
    w!(buf, "    }}\n");
    blank_line(buf);

    w!(buf, "    #[allow(dead_code)]\n");
    w!(buf, "    pub fn from_vec(args: Vec<std::ffi::OsString>) -> xflags::Result<Self> {{\n");
    w!(buf, "        Self::from_vec_(args)\n");
    w!(buf, "    }}\n");
    w!(buf, "}}\n");
}

fn emit_impls(buf: &mut String, xflags: &ast::XFlags) -> () {
    w!(buf, "impl {} {{\n", camel(&xflags.cmd.name));
    w!(buf, "    fn from_env_() -> xflags::Result<Self> {{\n");
    w!(buf, "        let mut p = xflags::rt::Parser::new_from_env();\n");
    w!(buf, "        Self::parse_(&mut p)\n");
    w!(buf, "    }}\n");
    w!(buf, "    fn from_vec_(args: Vec<std::ffi::OsString>) -> xflags::Result<Self> {{\n");
    w!(buf, "        let mut p = xflags::rt::Parser::new(args);\n");
    w!(buf, "        Self::parse_(&mut p)\n");
    w!(buf, "    }}\n");
    w!(buf, "}}\n");
    blank_line(buf);
    emit_newstyle_impl(buf, &xflags.cmd)
    // emit_impls_rec(buf, &xflags.cmd)
}

fn emit_newstyle_impl(buf: &mut String, cmd: &ast::Cmd) {
    w!(buf, "impl {} {{\n", camel(&cmd.name));
    w!(buf, "fn parse_(p_: &mut xflags::rt::Parser) -> xflags::Result<Self> {{\n");
    w!(buf, "#![allow(non_snake_case)]\n");

    let mut prefix = String::new();
    emit_locals_rec(buf, &mut prefix, cmd);
    blank_line(buf);
    w!(buf, "let mut state_ = 0u8;\n");
    w!(buf, "while let Some(arg_) = p_.pop_flag() {{\n");
    w!(buf, "match arg_ {{\n");
    {
        w!(buf, "Ok(flag_) => match (state_, flag_.as_str()) {{\n");
        emit_match_flag_rec(buf, &mut prefix, cmd);
        w!(buf, "_ => return Err(p_.unexpected_flag(&flag_)),\n");
        w!(buf, "}}\n");

        w!(buf, "Err(arg_) => match (state_, arg_.to_str().unwrap_or(\"\")) {{\n");
        emit_match_arg_rec(buf, &mut prefix, cmd);
        w!(buf, "_ => return Err(p_.unexpected_arg(arg_)),\n");
        w!(buf, "}}\n");
    }
    w!(buf, "}}\n");
    w!(buf, "}}\n");
    emit_default_transitions(buf, cmd);

    w!(buf, "Ok(");
    emit_record_rec(buf, &mut prefix, cmd);
    w!(buf, ")");

    w!(buf, "}}\n");
    w!(buf, "}}\n");
}

fn emit_locals_rec(buf: &mut String, prefix: &mut String, cmd: &ast::Cmd) {
    for flag in &cmd.flags {
        w!(buf, "let mut {prefix}{} = Vec::new();\n", flag.ident());
    }
    for arg in &cmd.args {
        w!(buf, "let mut {prefix}{} = (false, Vec::new());\n", arg.val.ident());
    }
    for cmd in &cmd.subcommands {
        let l = prefix.len();
        prefix.push_str(&snake(&cmd.name));
        prefix.push_str("__");
        emit_locals_rec(buf, prefix, cmd);
        prefix.truncate(l);
    }
}

fn emit_match_flag_rec(buf: &mut String, prefix: &mut String, cmd: &ast::Cmd) {
    for flag in &cmd.flags {
        w!(buf, "({}, ", cmd.idx);
        w!(buf, "\"--{}\"", flag.name);
        if let Some(short) = &flag.short {
            w!(buf, "| \"-{}\"", short);
        }
        w!(buf, ") => {prefix}{}.push(", flag.ident());
        match &flag.val {
            Some(val) => match &val.ty {
                ast::Ty::OsString | ast::Ty::PathBuf => {
                    w!(buf, "p_.next_value(&flag_)?.into()")
                }
                ast::Ty::FromStr(ty) => {
                    w!(buf, "p_.next_value_from_str::<{ty}>(&flag_)?")
                }
            },
            None => w!(buf, "()"),
        }
        w!(buf, "),");
    }
    if let Some(sub) = cmd.default_subcommand() {
        w!(buf, "({}, _) => {{ p_.push_back(Ok(flag_)); state_ = {}; }}", cmd.idx, sub.idx);
    }
    for cmd in &cmd.subcommands {
        let l = prefix.len();
        prefix.push_str(&snake(&cmd.name));
        prefix.push_str("__");
        emit_match_flag_rec(buf, prefix, cmd);
        prefix.truncate(l);
    }
}

fn emit_match_arg_rec(buf: &mut String, prefix: &mut String, cmd: &ast::Cmd) {
    for sub in cmd.named_subcommands() {
        w!(buf, "({}, \"{}\") => state_ = {},\n", cmd.idx, sub.name, sub.idx);
    }
    if !cmd.args.is_empty() || cmd.has_subcommands() {
        w!(buf, "({}, _) => {{\n", cmd.idx);
        for arg in &cmd.args {
            let done = match arg.arity {
                ast::Arity::Optional | ast::Arity::Required => "done_ @ ",
                ast::Arity::Repeated => "",
            };
            w!(buf, "if let ({done}false, buf_) = &mut {prefix}{} {{\n", arg.val.ident());
            w!(buf, "buf_.push(");
            match &arg.val.ty {
                ast::Ty::OsString | ast::Ty::PathBuf => {
                    w!(buf, "arg_.into()")
                }
                ast::Ty::FromStr(ty) => {
                    w!(buf, "p_.value_from_str::<{ty}>(\"{}\", arg_)?", arg.val.name);
                }
            }
            w!(buf, ");\n");
            match arg.arity {
                ast::Arity::Optional | ast::Arity::Required => {
                    w!(buf, "*done_ = true;\n");
                }
                ast::Arity::Repeated => (),
            }
            w!(buf, "continue;\n");
            w!(buf, "}}\n");
        }

        if let Some(sub) = cmd.default_subcommand() {
            w!(buf, "p_.push_back(Err(arg_)); state_ = {};", sub.idx);
        } else {
            w!(buf, "return Err(p_.unexpected_arg(arg_));");
        }

        w!(buf, "}}\n");
    }

    for cmd in &cmd.subcommands {
        let l = prefix.len();
        prefix.push_str(&snake(&cmd.name));
        prefix.push_str("__");
        emit_match_arg_rec(buf, prefix, cmd);
        prefix.truncate(l);
    }
}

fn emit_record_rec(buf: &mut String, prefix: &mut String, cmd: &ast::Cmd) {
    w!(buf, "{} {{\n", camel(&cmd.name));

    for flag in &cmd.flags {
        w!(buf, "{}: ", flag.ident());
        match &flag.val {
            Some(_val) => match flag.arity {
                ast::Arity::Optional => {
                    w!(buf, "p_.optional(\"--{}\", {prefix}{})?", flag.name, flag.ident())
                }
                ast::Arity::Required => {
                    w!(buf, "p_.required(\"--{}\", {prefix}{})?", flag.name, flag.ident())
                }
                ast::Arity::Repeated => w!(buf, "{prefix}{}", flag.ident()),
            },
            None => match flag.arity {
                ast::Arity::Optional => {
                    w!(buf, "p_.optional(\"--{}\", {prefix}{})?.is_some()", flag.name, flag.ident())
                }
                ast::Arity::Required => {
                    w!(buf, "p_.required(\"--{}\", {prefix}{})?", flag.name, flag.ident())
                }
                ast::Arity::Repeated => w!(buf, "{prefix}{}.len() as u32", flag.ident()),
            },
        }
        w!(buf, ",\n");
    }
    for arg in &cmd.args {
        let val = &arg.val;
        w!(buf, "{}: ", val.ident());
        match arg.arity {
            ast::Arity::Optional => {
                w!(buf, "p_.optional(\"{}\", {prefix}{}.1)?", val.name, val.ident())
            }
            ast::Arity::Required => {
                w!(buf, "p_.required(\"{}\", {prefix}{}.1)?", val.name, val.ident())
            }
            ast::Arity::Repeated => w!(buf, "{prefix}{}.1", val.ident()),
        }
        w!(buf, ",\n");
    }
    if cmd.has_subcommands() {
        w!(buf, "subcommand: match state_ {{\n");
        for sub in &cmd.subcommands {
            emit_ids_rec(buf, sub);
            w!(buf, " => {}Cmd::{}(", camel(&cmd.name), camel(&sub.name));
            let l = prefix.len();
            prefix.push_str(&snake(&sub.name));
            prefix.push_str("__");
            emit_record_rec(buf, prefix, sub);
            prefix.truncate(l);
            w!(buf, "),\n");
        }
        w!(buf, "_ => return Err(p_.subcommand_required()),");
        w!(buf, "}}\n");
    }

    w!(buf, "}}");
}

fn emit_ids_rec(buf: &mut String, cmd: &ast::Cmd) {
    if cmd.has_subcommands() {
        for sub in &cmd.subcommands {
            emit_ids_rec(buf, sub)
        }
    } else {
        w!(buf, "| {}", cmd.idx)
    }
}

fn emit_default_transitions(buf: &mut String, cmd: &ast::Cmd)  {
    if let Some(sub) = cmd.default_subcommand() {
        w!(buf, "state_ = if state_ == {} {{ {} }} else {{ state_ }};", cmd.idx, sub.idx);
    }
    for sub in &cmd.subcommands {
        emit_default_transitions(buf, sub);
    }
}

fn emit_help(buf: &mut String, xflags: &ast::XFlags) {
    w!(buf, "impl {} {{\n", xflags.cmd.ident());

    let help = {
        let mut buf = String::new();
        help_rec(&mut buf, "", &xflags.cmd);
        buf
    };
    let help = format!("{:?}", help);
    let help = help.replace("\\n", "\n").replacen("\"", "\"\\\n", 1);

    w!(buf, "const HELP_: &'static str = {};", help);
    w!(buf, "}}\n");
}

fn write_lines_indented(buf: &mut String, multiline_str: &str, indent: usize) {
    for line in multiline_str.split('\n').map(str::trim_end) {
        if line.is_empty() {
            w!(buf, "\n")
        } else {
            w!(buf, "{blank:indent$}{}\n", line, indent = indent, blank = "");
        }
    }
}

fn help_rec(buf: &mut String, prefix: &str, cmd: &ast::Cmd) {
    w!(buf, "{}{}\n", prefix, cmd.name);
    if let Some(doc) = &cmd.doc {
        write_lines_indented(buf, doc, 2);
    }
    let indent = if prefix.is_empty() { "" } else { "  " };

    let args = cmd.args_with_default();
    if !args.is_empty() {
        blank_line(buf);
        w!(buf, "{}ARGS:\n", indent);

        let mut blank = "";
        for arg in &args {
            w!(buf, "{}", blank);
            blank = "\n";

            let (l, r) = match arg.arity {
                ast::Arity::Optional => ("[", "]"),
                ast::Arity::Required => ("<", ">"),
                ast::Arity::Repeated => ("<", ">..."),
            };
            w!(buf, "    {}{}{}\n", l, arg.val.name, r);
            if let Some(doc) = &arg.doc {
                write_lines_indented(buf, doc, 6)
            }
        }
    }

    let flags = cmd.flags_with_default();
    if !flags.is_empty() {
        blank_line(buf);
        w!(buf, "{}OPTIONS:\n", indent);

        let mut blank = "";
        for flag in &flags {
            w!(buf, "{}", blank);
            blank = "\n";

            let short = flag.short.as_ref().map(|it| format!("-{}, ", it)).unwrap_or_default();
            let value = flag.val.as_ref().map(|it| format!(" <{}>", it.name)).unwrap_or_default();
            w!(buf, "    {}--{}{}\n", short, flag.name, value);
            if let Some(doc) = &flag.doc {
                write_lines_indented(buf, doc, 6);
            }
        }
    }

    let subcommands = cmd.named_subcommands();
    if !subcommands.is_empty() {
        if prefix.is_empty() {
            blank_line(buf);
            w!(buf, "SUBCOMMANDS:");
        }

        let prefix = format!("{}{} ", prefix, cmd.name);
        for sub in subcommands {
            blank_line(buf);
            blank_line(buf);
            help_rec(buf, &prefix, sub);
        }
    }
}

impl ast::Cmd {
    fn ident(&self) -> String {
        camel(&self.name)
    }
    fn cmd_enum_ident(&self) -> String {
        format!("{}Cmd", self.ident())
    }
    fn has_subcommands(&self) -> bool {
        !self.subcommands.is_empty()
    }
    fn named_subcommands(&self) -> &[ast::Cmd] {
        let start = if self.default { 1 } else { 0 };
        &self.subcommands[start..]
    }
    fn default_subcommand(&self) -> Option<&ast::Cmd> {
        if self.default {
            self.subcommands.first()
        } else {
            None
        }
    }
    fn args_with_default(&self) -> Vec<&ast::Arg> {
        let mut res = self.args.iter().collect::<Vec<_>>();
        if let Some(sub) = self.default_subcommand() {
            res.extend(sub.args_with_default());
        }
        res
    }
    fn flags_with_default(&self) -> Vec<&ast::Flag> {
        let mut res = self.flags.iter().collect::<Vec<_>>();
        if let Some(sub) = self.default_subcommand() {
            res.extend(sub.flags_with_default())
        }
        res
    }
}

impl ast::Flag {
    fn ident(&self) -> String {
        snake(&self.name)
    }
}

impl ast::Val {
    fn ident(&self) -> String {
        snake(&self.name)
    }
}

fn blank_line(buf: &mut String) {
    w!(buf, "\n");
}

fn camel(s: &str) -> String {
    s.split('-').map(first_upper).collect()
}

fn first_upper(s: &str) -> String {
    s.chars()
        .next()
        .map(|it| it.to_ascii_uppercase())
        .into_iter()
        .chain(s.chars().skip(1))
        .collect()
}

fn snake(s: &str) -> String {
    s.replace('-', "_")
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::Write,
        path::Path,
        process::{Command, Stdio},
    };

    fn reformat(text: String) -> Option<String> {
        let mut rustfmt =
            Command::new("rustfmt").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
        let mut stdin = rustfmt.stdin.take().unwrap();
        stdin.write_all(text.as_bytes()).unwrap();
        drop(stdin);
        let out = rustfmt.wait_with_output().unwrap();
        let res = String::from_utf8(out.stdout).unwrap();
        if res.is_empty() {
            None
        } else {
            Some(res)
        }
    }

    fn update_on_disk_if_different(file: &Path, new_contents: String) -> bool {
        let old_contents = fs::read_to_string(file).unwrap_or_default();
        if old_contents.trim() == new_contents.trim() {
            return false;
        }
        fs::write(file, new_contents).unwrap();
        true
    }

    #[test]
    fn gen_it() {
        let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/it");

        let mut did_update = false;
        for entry in fs::read_dir(test_dir.join("src")).unwrap() {
            let entry = entry.unwrap();

            let text = fs::read_to_string(entry.path()).unwrap();
            let mut lines = text.lines().collect::<Vec<_>>();
            lines.pop();
            lines.remove(0);
            let text = lines.join("\n");

            let res = crate::compile(&text);
            let fmt = reformat(res.clone());

            let code = format!(
                "#![allow(unused)]\nuse std::{{ffi::OsString, path::PathBuf}};\n\n{}",
                fmt.as_deref().unwrap_or(&res)
            );

            let name = entry.file_name();
            did_update |= update_on_disk_if_different(&test_dir.join(name), code);

            if fmt.is_none() {
                panic!("syntax error");
            }
        }
        if did_update {
            panic!("generated output changed")
        }
    }
}
