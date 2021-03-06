mod ast;
mod parse;
mod emit;
mod update;

#[cfg(not(test))]
#[proc_macro]
pub fn xflags(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let cmd = parse::parse(ts).unwrap();
    let text = emit::emit(&cmd);
    text.parse().unwrap()
}

#[cfg(test)]
pub fn compile(src: &str) -> String {
    use proc_macro2::TokenStream;

    let ts = src.parse::<TokenStream>().unwrap();
    let cmd = parse::parse(ts).unwrap();
    emit::emit(&cmd)
}
