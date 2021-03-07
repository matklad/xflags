xflags! {
    cmd rust-analyzer {
        repeated -v, --verbose

        cmd server {
            optional --dir path:PathBuf
            default cmd launch {
                optional --log
            }
            cmd watch {
            }
        }

        cmd analysis-stats
            required path: PathBuf
        {
            optional --parallel
        }
    }
}
