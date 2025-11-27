# [FuzzyDate][docsrs]: Date Input for Humans

[![FuzzyDate Github Actions][gh-image]][gh-checks]
[![FuzzyDate on crates.io][cratesio-image]][cratesio]
[![FuzzyDate on docs.rs][docsrs-image]][docsrs]

[gh-image]: https://github.com/DevinVS/fuzzydate/actions/workflows/tests.yml/badge.svg
[gh-checks]: https://github.com/DevinVS/fuzzydate/actions/workflows/tests.yml
[cratesio-image]: https://img.shields.io/crates/v/fuzzydate.svg
[cratesio]: https://crates.io/crates/fuzzydate
[docsrs-image]: https://docs.rs/fuzzydate/badge.svg
[docsrs]: https://docs.rs/fuzzydate

A flexible date parser library for Rust.

## Usage

Put this in your `Cargo.toml`:

```toml
[dependencies]
fuzzydate = "0.3"
```

See the [fuzzydate crate README](fuzzydate/README.md) for more information.

## Example

```rust
use fuzzydate::parse;

fn main() {
    let input = "five days after this friday";
    let date = parse(input).unwrap();
    println!("{:?}", date);
}
```

## fuzzydate-cli

This project also provides a small command-line interface for the `fuzzydate` library. See the crate-level READMEs for more information and examples:

* Library (fuzzydate): [fuzzydate/README.md](fuzzydate/README.md)
* CLI (fuzzydate-cli): [fuzzydate-cli/README.md](fuzzydate-cli/README.md)

The CLI parses human-friendly date expressions and prints a `chrono`-formatted datetime. For example:

```sh
$ fuzzydate 5 minutes after friday at noon
2025-11-28T12:05:00-08:00
```

See `fuzzydate --help` for options and usage.

### Installation

```sh
cargo install fuzzydate-cli --locked
```
