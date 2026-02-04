# fuzzydate-cli

[crates.io]: https://crates.io/crates/fuzzydate-cli
[docs.rs]: https://docs.rs/fuzzydate
[repo]: https://github.com/DevinVS/fuzzydate
[license]: https://github.com/DevinVS/fuzzydate/blob/main/LICENSE

[![fuzzydate-cli on crates.io](https://img.shields.io/crates/v/fuzzydate-cli.svg)](https://crates.io/crates/fuzzydate-cli)

This is a command line interface to the `fuzzydate` library, which parses human-friendly
date/time phrases (for example: "five days after this Friday", "tomorrow at noon", "3 weeks ago")
and prints timestamps in standard formats.

## Quick links
* [crate][crates.io]
* [library documentation][docs.rs]
* [repo][repo]
* [license][license]

## Install

```sh
cargo install fuzzydate-cli --locked
```

## Usage

Fuzzydate parses human-friendly phrases into timestamps. The current time (or `--relative-to`, if
passed), is used for any values not specified by the phrase and for any operations that are relative
to an existing time, like `5 days ago`. For full grammar, see the [library documentation][docs.rs].
See `fuzzydate --help` for information on supported options.

Simple phrase:
```sh
$ fuzzydate 5 minutes after friday at noon
2025-11-28T12:05:00-08:00
```

Change output format:
```sh
$ fuzzydate -f "%Y-%m-%d %H:%M" tomorrow at 9:30am
2025-11-29 09:30
```

Change time used as current time:
```sh
$ fuzzydate --relative-to 2025-11-27T08:00:00-08:00 tomorrow at noon
2025-11-28T12:00:00-08:00
```
