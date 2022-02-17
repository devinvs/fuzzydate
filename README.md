# [FuzzyDate][docsrs]: Date Input for Humans
============================================

[![FuzzyDate Github Actions][gh-image]][gh-checks]
[![FuzzyDate on crates.io][cratesio-image]][cratesio]
[![FuzzyDate on docs.rs][docsrs-image]][docsrs]

[gh-image]: https://github.com/DevinVS/FuzzyDate/actions/workflows/tests.yml/badge.svg
[gh-checks]: https://github.com/DevinVS/FuzzyDate/actions?query=workflow%3Atests
[cratesio-image]: https://img.shields.io/crates/v/fuzzydate.svg
[cratesio]: https://crates.io/crates/fuzzydate
[docsrs-image]: https://docs.rs/fuzzydate/badge.svg
[docsrs]: https://docs.rs/fuzzydate

A flexible date parser library for Rust.

## Usage

Put this in your `Cargo.tom`:

```toml
[dependencies]
fuzzydate = "0.1"
```

## Example

```rust
use fuzzydate::parse;

fn main() {
    let input = "five days after this friday";
    let date = parse(input).unwrap();
    println!("{:?}", date);
}
```
