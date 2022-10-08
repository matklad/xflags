use std::path::PathBuf;

fn main() {
    let flags = xflags::parse_or_exit! {
        /// Remove directories and their contents recursively.
        optional -r,--recursive
        /// File or directory to remove
        required path: PathBuf
    };

    println!(
        "removing {}{}",
        flags.path.display(),
        if flags.recursive { "recursively" } else { "" },
    )
}
