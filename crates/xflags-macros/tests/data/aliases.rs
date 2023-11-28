xflags! {
    /// commands with different aliases
    cmd alias-cmd {
        /// And even an aliased subcommand!
        cmd sub s {
            /// Little sanity check to see if this still works as intended
            optional -c, --count count: usize
        }
        cmd this one has a lot of aliases {}
    }
}
