xflags! {
    /// Does stuff
    cmd helpful
        /// With an arg.
        optional src: PathBuf
    {
        /// And a switch.
        required -s, --switch

        /// And even a subcommand!
        cmd sub {
        }
    }
}
