# Changelog

## Unreleased

-

## 0.3.2

- Support command aliases.
- Fix unused mut warning for empty commands.

## 0.3.1

- Better align with [fuchsia CLI spec](https://fuchsia.dev/fuchsia-src/development/api/cli#command_line_arguments):

  * values can begin with `-`: `--takes-value --i-am-the-value`
  * support `--` to delimit positional arguments: `cargo run -- --not-an-option`
