mod empty;
mod smoke;
mod repeated_pos;
mod subcommands;
mod help;

use std::{ffi::OsString, fmt};

use expect_test::{expect, Expect};

fn check<F, A>(f: F, args: &str, expect: Expect)
where
    F: FnOnce(Vec<OsString>) -> xflags::Result<A>,
    A: fmt::Debug,
{
    let args = args.split_ascii_whitespace().map(OsString::from).collect::<Vec<_>>();
    let res = f(args);
    match res {
        Ok(args) => {
            expect.assert_debug_eq(&args);
        }
        Err(err) => {
            expect.assert_eq(&err.to_string());
        }
    }
}

#[test]
fn empty() {
    check(
        empty::Empty::from_vec,
        "",
        expect![[r#"
        Empty
    "#]],
    )
}

#[test]
fn smoke() {
    check(
        smoke::RustAnalyzer::from_vec,
        "-n 92 .",
        expect![[r#"
            RustAnalyzer {
                workspace: ".",
                jobs: None,
                log_file: None,
                verbose: 0,
                number: 92,
                data: [],
                emoji: false,
            }
        "#]],
    );
    check(
        smoke::RustAnalyzer::from_vec,
        "-n 92 -v --verbose -v --data 0xDEAD --log-file /tmp/log.txt --data 0xBEEF .",
        expect![[r#"
            RustAnalyzer {
                workspace: ".",
                jobs: None,
                log_file: Some(
                    "/tmp/log.txt",
                ),
                verbose: 3,
                number: 92,
                data: [
                    "0xDEAD",
                    "0xBEEF",
                ],
                emoji: false,
            }
        "#]],
    );

    check(
        smoke::RustAnalyzer::from_vec,
        "-n 92 --werbose",
        expect!["Unknown flag: `--werbose`. Use `help` for more information"],
    );
    check(
        smoke::RustAnalyzer::from_vec,
        "",
        expect!["Flag is required: `--number`. Use `help` for more information"],
    );
    check(
        smoke::RustAnalyzer::from_vec,
        ".",
        expect!["Flag is required: `--number`. Use `help` for more information"],
    );
    check(smoke::RustAnalyzer::from_vec, "-n", expect![[r#"expected a value for `-n`"#]]);
    check(
        smoke::RustAnalyzer::from_vec,
        "-n 92",
        expect!["Flag is required: `workspace`. Use `help` for more information"],
    );
    check(
        smoke::RustAnalyzer::from_vec,
        "-n lol",
        expect!["Can't parse `-n`, invalid digit found in string"],
    );
    check(
        smoke::RustAnalyzer::from_vec,
        "-n 1 -n 2 .",
        expect!["Flag specified more than once: `--number`"],
    );
    check(
        smoke::RustAnalyzer::from_vec,
        "-n 1 . 92 lol",
        expect!["Unknown command: `lol`. Use `help` for more information"],
    );
    check(
        smoke::RustAnalyzer::from_vec,
        "-n 1 . --emoji --emoji",
        expect!["Flag specified more than once: `--emoji`"],
    );
}

#[test]
fn repeated_argument() {
    check(
        repeated_pos::RepeatedPos::from_vec,
        "a 11 c d e f",
        expect![[r#"
            RepeatedPos {
                a: "a",
                b: Some(
                    11,
                ),
                c: Some(
                    "c",
                ),
                rest: [
                    "d",
                    "e",
                    "f",
                ],
            }
        "#]],
    );
}

#[test]
fn subcommands() {
    check(
        subcommands::RustAnalyzer::from_vec,
        "server",
        expect![[r#"
            RustAnalyzer {
                verbose: 0,
                subcommand: Server(
                    Server {
                        dir: None,
                        subcommand: Launch(
                            Launch {
                                log: false,
                            },
                        ),
                    },
                ),
            }
        "#]],
    );

    check(
        subcommands::RustAnalyzer::from_vec,
        "server --dir . --log",
        expect![[r#"
            RustAnalyzer {
                verbose: 0,
                subcommand: Server(
                    Server {
                        dir: Some(
                            ".",
                        ),
                        subcommand: Launch(
                            Launch {
                                log: true,
                            },
                        ),
                    },
                ),
            }
        "#]],
    );

    check(
        subcommands::RustAnalyzer::from_vec,
        "server watch",
        expect![[r#"
            RustAnalyzer {
                verbose: 0,
                subcommand: Server(
                    Server {
                        dir: None,
                        subcommand: Watch(
                            Watch,
                        ),
                    },
                ),
            }
        "#]],
    );

    check(
        subcommands::RustAnalyzer::from_vec,
        "-v analysis-stats . --parallel",
        expect![[r#"
            RustAnalyzer {
                verbose: 1,
                subcommand: AnalysisStats(
                    AnalysisStats {
                        path: ".",
                        parallel: true,
                    },
                ),
            }
        "#]],
    );

    check(
        subcommands::RustAnalyzer::from_vec,
        "",
        expect!["A subcommand is required. Use `help` for more information"],
    );
}

#[test]
fn subcommand_flag_inheritance() {
    check(
        subcommands::RustAnalyzer::from_vec,
        "server watch --verbose --dir .",
        expect![[r#"
            RustAnalyzer {
                verbose: 1,
                subcommand: Server(
                    Server {
                        dir: Some(
                            ".",
                        ),
                        subcommand: Watch(
                            Watch,
                        ),
                    },
                ),
            }
        "#]],
    );
    check(
        subcommands::RustAnalyzer::from_vec,
        "analysis-stats --verbose --dir .",
        expect!["Unknown flag: `--dir`. Use `help` for more information"],
    );
    check(
        subcommands::RustAnalyzer::from_vec,
        "--dir . server",
        expect!["Unknown flag: `--dir`. Use `help` for more information"],
    );
}

#[test]
fn edge_cases() {
    check(
        subcommands::RustAnalyzer::from_vec,
        "server --dir --log",
        expect![[r#"
            RustAnalyzer {
                verbose: 0,
                subcommand: Server(
                    Server {
                        dir: Some(
                            "--log",
                        ),
                        subcommand: Launch(
                            Launch {
                                log: false,
                            },
                        ),
                    },
                ),
            }
        "#]],
    );
    check(
        subcommands::RustAnalyzer::from_vec,
        "server --dir -- --log",
        expect![[r#"
            RustAnalyzer {
                verbose: 0,
                subcommand: Server(
                    Server {
                        dir: Some(
                            "--",
                        ),
                        subcommand: Launch(
                            Launch {
                                log: true,
                            },
                        ),
                    },
                ),
            }
        "#]],
    );
    check(
        subcommands::RustAnalyzer::from_vec,
        "-- -v server",
        expect!["Unknown command: `-v`. Use `help` for more information"],
    );
    check(
        repeated_pos::RepeatedPos::from_vec,
        "pos 1 prog -j",
        expect!["Unknown flag: `-j`. Use `help` for more information"],
    );
    check(
        repeated_pos::RepeatedPos::from_vec,
        "pos 1 -- prog -j",
        expect![[r#"
            RepeatedPos {
                a: "pos",
                b: Some(
                    1,
                ),
                c: Some(
                    "prog",
                ),
                rest: [
                    "-j",
                ],
            }
        "#]],
    );
}
