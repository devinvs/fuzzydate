# fuzzydate

[![fuzzydate on crates.io](https://img.shields.io/crates/v/fuzzydate-cli.svg)](https://crates.io/crates/fuzzydate) [![fuzzydate on docs.rs](https://docs.rs/fuzzydate/badge.svg)](https://docs.rs/fuzzydate)

[crates.io]: https://crates.io/crates/fuzzydate
[docs.rs]: https://docs.rs/fuzzydate
[repo]: https://github.com/DevinVS/fuzzydate
[license]: https://github.com/DevinVS/fuzzydate/blob/main/LICENSE

fuzzydate parses human-friendly date/time phrases (for example: "five days after this Friday", "tomorrow at noon", "3 weeks ago") and returns `chrono` datetimes. The current time (or a passed alternative), is used for any values not specified by the phrase and for any operations that are relative to an existing time, like `5 days ago`. By default, fuzzydate uses the current time and system timezone.

## Quick links
* [crate][crates.io]
* [documentation][docs.rs]
* [repo][repo]
* [license][license]

## Install

Add to your `Cargo.toml`:

```toml
[dependencies]
fuzzydate = "0.3"
```

## Examples

For full grammar reference, see [the documentation][docs.rs].

```rust
use fuzzydate::parse;

fn main() -> Result<(), fuzzydate::Error> {
    let dt = parse("five days after this friday")?;
    println!("{:?}", dt);
    Ok(())
}
```

Additional functions are available to parse datetimes with an alternative current time and parse in a timezone other than the system timezone:

```rust
use fuzzydate::aware_parse;
use chrono::offset::Utc;
use chrono::prelude::*;

fn main() -> Result<(), fuzzydate::Error> {
    let dt = aware_parse("tomorrow at noon", Utc::now(), Utc)?;
    println!("{}", dt);
    Ok(())
}
```
