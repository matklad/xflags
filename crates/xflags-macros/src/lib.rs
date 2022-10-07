mod ast;
mod parse;
mod emit;
mod update;

#[proc_macro]
pub fn xflags(_ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Stub out the code, but let rust-analyzer resolve the invocation
    #[cfg(not(test))]
    {
        let text = match parse::xflags(_ts) {
            Ok(cmd) => emit::emit(&cmd),
            Err(err) => format!("compile_error!(\"invalid flags syntax, {err}\");"),
        };
        text.parse().unwrap()
    }
    #[cfg(test)]
    unimplemented!()
}

#[proc_macro]
pub fn parse_or_exit(_ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Stub out the code, but let rust-analyzer resolve the invocation
    #[cfg(not(test))]
    {
        let text = match parse::parse_or_exit(_ts) {
            Ok(cmd) => emit::emit(&cmd),
            Err(err) => format!("compile_error!(\"invalid flags syntax, {err}\")"),
        };
        text.parse().unwrap()
    }
    #[cfg(test)]
    {
        let _ = parse::parse_or_exit;
        unimplemented!();
    }
}

#[cfg(test)]
pub fn compile(src: &str) -> String {
    use proc_macro2::TokenStream;

    let ts = src.parse::<TokenStream>().unwrap();
    let cmd = parse::xflags(ts).unwrap();
    emit::emit(&cmd)
}
