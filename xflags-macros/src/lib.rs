mod ast;
mod parse;
mod emit;

#[cfg(not(test))]
#[proc_macro]
pub fn args(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let cmd = parse::parse(ts).unwrap();
    let text = emit::emit(&cmd);
    text.parse().unwrap()
}

#[cfg(not(test))]
#[proc_macro]
pub fn args_parser(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let cmd = parse::parse(ts).unwrap();
    let text = emit::emit_parser(&cmd);
    text.parse().unwrap()
}

#[cfg(test)]
pub fn compile(src: &str) -> String {
    use proc_macro2::TokenStream;

    let ts = src.parse::<TokenStream>().unwrap();
    let cmd = parse::parse(ts).unwrap();
    emit::emit(&cmd)
}
