use std::path::PathBuf;

fn main() {
    let flags = xflags::parse_or_exit! {
        optional -r,--recursive
        required path: PathBuf
    };

    println!(
        "removing {}{}",
        flags.path.display(),
        if flags.recursive { "recursively" } else { "" },
    )
}
