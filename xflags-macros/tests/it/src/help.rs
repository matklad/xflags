xflags! {
    /// Does stuff
    cmd helpful
        /// With an arg.
        optional src: PathBut
    {
        /// And a switch.
        required -s, --switch

        /// And even a subcommand!
        cmd sub {
        }
    }
}
