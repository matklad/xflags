use crate::{ast, update};

use std::{env, fmt::Write, path::Path};

macro_rules! w {
    ($($tt:tt)*) => {{ let _ = write!($($tt)*); }};
}

pub(crate) fn emit(xflags: &ast::XFlags) -> String {
    let mut buf = String::new();

    if xflags.is_anon() {
        w!(buf, "{{\n");
    }

    emit_cmd(&mut buf, &xflags.cmd);
    blank_line(&mut buf);
    emit_api(&mut buf, xflags);

    if !xflags.is_anon() && env::var("UPDATE_XFLAGS").is_ok() {
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
    emit_impls(&mut buf, xflags);
    emit_help(&mut buf, xflags);

    if xflags.is_anon() {
        w!(buf, "Flags::from_env_or_exit()");
        w!(buf, "}}\n");
    }

    buf
}

fn emit_cmd(buf: &mut String, cmd: &ast::Cmd) {
    w!(buf, "#[derive(Debug)]\n");
    w!(buf, "pub struct {}", cmd.ident());
    let flags = cmd.flags.iter().filter(|it| !it.is_help()).collect::<Vec<_>>();
    if cmd.args.is_empty() && flags.is_empty() && cmd.subcommands.is_empty() {
        w!(buf, ";\n");
        return;
    }
    w!(buf, " {{\n");

    for arg in &cmd.args {
        let ty = gen_arg_ty(arg.arity, &arg.val.ty);
        w!(buf, "    pub {}: {ty},\n", arg.val.ident());
    }

    if !cmd.args.is_empty() && !flags.is_empty() {
        blank_line(buf);
    }

    for flag in &flags {
        let ty = gen_flag_ty(flag.arity, flag.val.as_ref().map(|it| &it.ty));
        w!(buf, "    pub {}: {ty},\n", flag.ident());
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
            let name = sub.ident();
            w!(buf, "    {name}({name}),\n");
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
    w!(buf, "impl {} {{\n", xflags.cmd.ident());

    w!(buf, "    #[allow(dead_code)]\n");
    w!(buf, "    pub fn from_env_or_exit() -> Self {{\n");
    w!(buf, "        Self::from_env_or_exit_()\n");
    w!(buf, "    }}\n");
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

fn emit_impls(buf: &mut String, xflags: &ast::XFlags) {
    w!(buf, "impl {} {{\n", xflags.cmd.ident());
    w!(buf, "    fn from_env_or_exit_() -> Self {{\n");
    w!(buf, "        Self::from_env_().unwrap_or_else(|err| err.exit())\n");
    w!(buf, "    }}\n");
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
    emit_parse(buf, &xflags.cmd)
}

fn emit_parse(buf: &mut String, cmd: &ast::Cmd) {
    w!(buf, "impl {} {{\n", cmd.ident());
    w!(buf, "fn parse_(p_: &mut xflags::rt::Parser) -> xflags::Result<Self> {{\n");
    w!(buf, "#![allow(non_snake_case, unused_mut)]\n");

    let mut prefix = String::new();
    emit_locals_rec(buf, &mut prefix, cmd);
    blank_line(buf);
    w!(buf, "let mut state_ = 0u8;\n");

    // No while loop needed for command with no items (clippy::never_loop)
    if cmd.args.len() + cmd.flags.len() + cmd.subcommands.len() <= 1 {
        w!(buf, "if let Some(arg_) = p_.pop_flag() {{\n");
    } else {
        w!(buf, "while let Some(arg_) = p_.pop_flag() {{\n");
    }

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
        if !flag.is_help() {
            w!(buf, "let mut {prefix}{} = Vec::new();\n", flag.ident());
        }
    }
    for arg in &cmd.args {
        w!(buf, "let mut {prefix}{} = (false, Vec::new());\n", arg.val.ident());
    }
    for sub in &cmd.subcommands {
        let l = sub.push_prefix(prefix);
        emit_locals_rec(buf, prefix, sub);
        prefix.truncate(l);
    }
}

fn emit_match_flag_rec(buf: &mut String, prefix: &mut String, cmd: &ast::Cmd) {
    w!(
        buf,
        "({}, \"--help\" | \"-h\") => return Err(p_.help(Self::HELP_{})),\n",
        cmd.idx,
        snake(prefix).to_uppercase()
    );
    for flag in cmd.flags.iter().filter(|f| !f.is_help()) {
        w!(buf, "(");
        emit_all_ids_rec(buf, cmd);
        w!(buf, ", \"--{}\"", flag.name);
        if let Some(short) = &flag.short {
            w!(buf, "| \"-{short}\"");
        }
        w!(buf, ") => ");
        w!(buf, "{prefix}{}.push(", flag.ident());
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
        w!(buf, "),\n");
    }
    if let Some(sub) = cmd.default_subcommand() {
        w!(buf, "({}, _) => {{ p_.push_back(Ok(flag_)); state_ = {}; }}", cmd.idx, sub.idx);
    }
    for sub in &cmd.subcommands {
        let l = sub.push_prefix(prefix);
        emit_match_flag_rec(buf, prefix, sub);
        prefix.truncate(l);
    }
}

fn emit_match_arg_rec(buf: &mut String, prefix: &mut String, cmd: &ast::Cmd) {
    for sub in cmd.named_subcommands() {
        let sub_match =
            sub.all_identifiers().map(|s| format!("\"{s}\"")).collect::<Vec<_>>().join(" | ");
        w!(buf, "({}, {}) => state_ = {},\n", cmd.idx, sub_match, sub.idx);
    }

    if cmd.args.is_empty() {
        // add `help` subcommand only if command takes no args to make sure it doesn't take precedence
        w!(
            buf,
            "({}, \"help\") => return Err(p_.help(Self::HELP_{})),\n",
            cmd.idx,
            snake(prefix).to_uppercase()
        );
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

    for sub in &cmd.subcommands {
        let l = sub.push_prefix(prefix);
        emit_match_arg_rec(buf, prefix, sub);
        prefix.truncate(l);
    }
}

fn emit_record_rec(buf: &mut String, prefix: &mut String, cmd: &ast::Cmd) {
    w!(buf, "{} {{\n", cmd.ident());

    for flag in &cmd.flags {
        if flag.is_help() {
            continue;
        }
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
            emit_leaf_ids_rec(buf, sub);
            w!(buf, " => {}::{}(", cmd.cmd_enum_ident(), sub.ident());
            let l = prefix.len();
            prefix.push_str(&snake(&sub.name));
            prefix.push_str("__");
            emit_record_rec(buf, prefix, sub);
            prefix.truncate(l);
            w!(buf, "),\n");
        }
        w!(buf, "_ => return Err(p_.subcommand_required())");
        w!(buf, "}}\n");
    }

    w!(buf, "}}");
}

fn emit_leaf_ids_rec(buf: &mut String, cmd: &ast::Cmd) {
    if cmd.has_subcommands() {
        for sub in &cmd.subcommands {
            emit_leaf_ids_rec(buf, sub)
        }
    } else {
        w!(buf, "| {}", cmd.idx)
    }
}

fn emit_all_ids_rec(buf: &mut String, cmd: &ast::Cmd) {
    w!(buf, "| {}", cmd.idx);
    for sub in &cmd.subcommands {
        emit_all_ids_rec(buf, sub)
    }
}

fn emit_default_transitions(buf: &mut String, cmd: &ast::Cmd) {
    if let Some(sub) = cmd.default_subcommand() {
        w!(buf, "state_ = if state_ == {} {{ {} }} else {{ state_ }};", cmd.idx, sub.idx);
    }
    for sub in &cmd.subcommands {
        emit_default_transitions(buf, sub);
    }
}

fn emit_help(buf: &mut String, xflags: &ast::XFlags) {
    w!(buf, "impl {} {{\n", xflags.cmd.ident());

    cmd_help_rec(buf, &xflags.cmd, "");

    w!(buf, "}}\n");
}

fn cmd_help_rec(buf: &mut String, cmd: &ast::Cmd, prefix: &str) {
    let mut help_buf = String::new();
    w!(help_buf, "Usage: {}", cmd.name);
    for arg in cmd.args_with_default() {
        let (l, r) = arg.arity.brackets();
        w!(help_buf, " {l}{}{r}", arg.val.name);
    }
    for flag in cmd.flags_with_default() {
        // <-f> doesn't make sense, if it has to be included it should just be -f
        let (l, r) = match flag.arity {
            ast::Arity::Required => ("", ""),
            _ => flag.arity.brackets(),
        };
        let f = flag.short.clone().unwrap_or_else(|| format!("-{}", flag.name));

        match &flag.val {
            Some(v) => w!(help_buf, " {l}-{f} <{}>{r}", v.name),
            None => w!(help_buf, " {l}-{f}{r}"),
        }
    }
    if cmd.has_subcommands() {
        w!(help_buf, " <COMMAND>")
    }
    if let Some(doc) = &cmd.doc {
        w!(help_buf, "\n\n{}\n", doc);
    }
    if !cmd.args_with_default().is_empty() {
        w!(help_buf, "\nArguments:\n");
        for arg in cmd.args_with_default() {
            let (l, r) = arg.arity.brackets();
            let pre_doc = format!("{l}{}{r}", arg.val.name);
            w!(help_buf, "  {:<20} {}\n", pre_doc, arg.doc.as_deref().unwrap_or(""));
        }
    }
    if !cmd.flags_with_default().is_empty() {
        w!(help_buf, "\nOptions:\n");
        for flag in cmd.flags_with_default() {
            let short = flag.short.as_ref().map(|it| format!("-{it}, ")).unwrap_or_default();
            let value = flag.val.as_ref().map(|it| format!(" <{}>", it.name)).unwrap_or_default();
            let pre_doc = format!("{short}--{}{value}", flag.name);
            w!(help_buf, "  {:<20} {}\n", pre_doc, flag.doc.as_deref().unwrap_or(""));
        }
    }
    w!(help_buf, "\nCommands:");
    for subcommand in &cmd.subcommands {
        w!(help_buf, "\n  {:<20} {}", subcommand.name, subcommand.doc.as_deref().unwrap_or(""));
        let prefix = format!("{}{}__", prefix, subcommand.name);
        cmd_help_rec(buf, subcommand, &prefix);
    }
    w!(help_buf, "\n  {:<20} ", "help");
    w!(help_buf, "Print this message or the help of the given subcommand(s)");
    w!(buf, "const HELP_{}: &'static str = \"{help_buf}\";\n", snake(prefix).to_uppercase());
}

impl ast::Cmd {
    fn ident(&self) -> String {
        if self.name.is_empty() {
            return "Flags".to_string();
        }
        camel(&self.name)
    }
    pub(crate) fn all_identifiers(&self) -> impl Iterator<Item = &String> {
        [&self.name].into_iter().chain(self.aliases.iter())
    }
    fn cmd_enum_ident(&self) -> String {
        format!("{}Cmd", self.ident())
    }
    fn push_prefix(&self, buf: &mut String) -> usize {
        let l = buf.len();
        buf.push_str(&snake(&self.name));
        buf.push_str("__");
        l
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

impl ast::Arity {
    fn brackets(&self) -> (&str, &str) {
        match self {
            ast::Arity::Optional => ("[", "]"),
            ast::Arity::Required => ("<", ">"),
            ast::Arity::Repeated => ("[", "]..."),
        }
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
        let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");

        let mut did_update = false;
        for entry in fs::read_dir(test_dir.join("data")).unwrap() {
            let entry = entry.unwrap();

            let text = fs::read_to_string(entry.path()).unwrap();
            let mut lines = text.lines().collect::<Vec<_>>();
            lines.pop();
            lines.remove(0);
            let text = lines.join("\n");

            let res = crate::compile(&text);
            let fmt = reformat(res.clone());

            let code = format!(
                "#[allow(unused)]\nuse std::{{ffi::OsString, path::PathBuf}};\n\n{}",
                fmt.as_deref().unwrap_or(&res)
            );

            let name = entry.file_name();
            did_update |= update_on_disk_if_different(&test_dir.join("it").join(name), code);

            if fmt.is_none() {
                panic!("syntax error");
            }
        }
        if did_update {
            panic!("generated output changed")
        }
    }
}
