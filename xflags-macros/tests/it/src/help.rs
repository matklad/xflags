xflags! {
    /// Does stuff
    ///
    /// Helpful stuff.
    cmd helpful
        /// With an arg.
        optional src: PathBuf
        /// Another arg.
        ///
        /// This time, we provide some extra info about the
        /// arg. Maybe some caveats, or what kinds of
        /// values are accepted.
        optional extra: String
        /// This arg will become an enum.
        optional channel: lts | stable | beta | nightly | dev
    {
        /// And a switch.
        required -s, --switch

        /// A list of allocators you happen to like.
        repeated --malloc mallocs: mimalloc | jemalloc | sys

        /// And even a subcommand!
        cmd sub {
            /// With an optional flag. This has a really long
            /// description which spans multiple lines.
            optional -f, --flag
        }
    }
}
