args! {
    /// LSP server for rust.
    cmd rust-analyzer
        required workspace: PathBuf
        /// Number of concurrent jobs.
        optional jobs: u32
    {
        /// Path to log file. By default, logs go to stderr.
        optional --log-file path: PathBuf
        repeated -v, --verbose
        required -n, --number n: u32
        repeated --data value: OsString
        optional --emoji
    }
}
