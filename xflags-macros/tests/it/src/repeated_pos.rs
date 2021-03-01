args! {
    cmd RepeatedPos
        required a: PathBuf
        optional b: u32
        optional c: OsString
        repeated rest: OsString
    {
    }
}
