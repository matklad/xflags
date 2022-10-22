# Changelog

## Unreleased

-

## 3.0.1

- Better align with [fuchsia CLI spec](https://fuchsia.dev/fuchsia-src/development/api/cli#command_line_arguments):

  * values can begin with `-`: `--takes-value --i-am-the-value`
  * support `--` to delimit positional arguments: `cargo run -- --not-an-option`
