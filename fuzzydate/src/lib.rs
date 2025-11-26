#![allow(clippy::needless_doctest_main)]
//! # FuzzyDate: Date Input for Humans
//!
//! A Parser which can turn a variety of input strings into a DateTime
//!
//! ## Usage
//!
//! Put this in your `Cargo.toml`:
//!
//! ```toml
//! fuzzydate = "0.2"
//! ```
//!
//! ## Example
//!
//! ```rust
//! use fuzzydate::parse;
//! use chrono::{NaiveDateTime};
//!
//! fn main() {
//!     let date_string = "Five days after 2/12/22 5:00 PM";
//!     let date = parse(date_string).unwrap();
//!     println!("{:?}", date);
//! }
//! ```
//!
//! Any relevant date time information not specified is assumed to be
//! the value of the current date time.
//!
//! ## Grammar
//! ```text
//! <datetime> ::= <time>
//!              | <time> , <date>
//!              | <time> <date>
//!              | <time> on <date>
//!              | <date>
//!              | <date> <time>
//!              | <date> , <time>
//!              | <date> at <time>
//!              | <duration> after <datetime>
//!              | <duration> from <datetime>
//!              | <duration> before <datetime>
//!              | now
//!
//! <date> ::= today
//!               | tomorrow
//!               | yesterday
//!               | <num> / <num> / <num>
//!               | <num> - <num> - <num>
//!               | <num> . <num> . <num>
//!               | <month> <num> <num>
//!               | <duration> ago              ; duration must be for a whole number of days
//!               | <duration> after <date>
//!               | <duration> from <date>
//!               | <duration> before <date>
//!               | <relative_specifier> <weekday>
//!               | <relative_specifier> <unit>
//!               | <weekday>
//!
//! <time> ::= <num>
//!          | <num>:<num>
//!          | <num>:<num> am
//!          | <num>:<num> pm
//!          | <num>
//!          | <num> am
//!          | <num> pm
//!          | <num> <num> am
//!          | <num> <num> pm
//!          | midnight
//!          | noon
//!
//! <duration> ::= <num> <unit>
//!              | <article> <unit>
//!              | <duration> and <duration>
//!
//! <article> ::= a
//!            | an
//!            | the
//!
//!
//! <relative_specifier> ::= this
//!                        | next
//!                        | last
//!
//! <weekday> ::= monday
//!             | tuesday
//!             | wednesday
//!             | thursday
//!             | friday
//!             | saturday
//!             | sunday
//!             | mon
//!             | tue
//!             | wed
//!             | thu
//!             | fri
//!             | sat
//!             | sun
//!
//! <month> ::= january
//!           | february
//!           | march
//!           | april
//!           | may
//!           | june
//!           | july
//!           | august
//!           | september
//!           | october
//!           | november
//!           | december
//!           | jan
//!           | feb
//!           | mar
//!           | apr
//!           | jun
//!           | jul
//!           | aug
//!           | sep
//!           | oct
//!           | nov
//!           | dec
//!
//! <unit> ::= day
//!          | days
//!          | week
//!          | weeks
//!          | hour
//!          | hours
//!          | minute
//!          | minutes
//!          | min
//!          | mins
//!          | month
//!          | months
//!          | year
//!          | years
//!
//! <num> ::= <num_triple> <num_triple_unit> and <num>
//!         | <num_triple> <num_triple_unit> <num>
//!         | <num_triple> <num_triple_unit>
//!         | <num_triple_unit> and <num>
//!         | <num_triple_unit> <num>
//!         | <num_triple_unit>
//!         | <num_triple>
//!         | NUM   ; number literal greater than or equal to 1000
//!
//! <num_triple> ::= <ones> hundred and <num_double>
//!                | <ones> hundred <num_double>
//!                | <ones> hundred
//!                | hundred and <num_double>
//!                | hundred <num_double>
//!                | hundred
//!                | <num_double>
//!                | NUM    ; number literal less than 1000 and greater than 99
//!
//! <num_triple_unit> ::= thousand
//!                     | million
//!                     | billion
//!
//! <num_double> ::= <ones>
//!                | <tens> - <ones>
//!                | <tens> <ones>
//!                | <tens>
//!                | <teens>
//!                | NUM    ; number literal less than 100 and greater than 19
//!
//! <tens> ::= twenty
//!          | thirty
//!          | forty
//!          | fifty
//!          | sixty
//!          | seventy
//!          | eighty
//!          | ninety
//!
//! <teens> ::= ten
//!           | eleven
//!           | twelve
//!           | thirteen
//!           | fourteen
//!           | fifteen
//!           | sixteen
//!           | seventeen
//!           | eighteen
//!           | nineteen
//!           | NUM     ; number literal less than 20 and greater than 9
//!
//! <ones> ::= one
//!          | two
//!          | three
//!          | four
//!          | five
//!          | six
//!          | seven
//!          | eight
//!          | nine
//!          | NUM      ; number literal less than 10
//! ```

mod ast;
mod lexer;

use chrono::{DateTime, Local, NaiveDateTime, NaiveTime, TimeZone};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("Invalid date: {0}")]
    /// The date is invalid,
    /// e.g. `"31st of February"`, `"December 32nd"`, `"32/13/2019"`
    InvalidDate(String),
    #[error("Unrecognized Token: {0}")]
    /// The lexer found a token that it doesn't recognize
    UnrecognizedToken(String),
    #[error("Unable to parse date")]
    /// The date _may_ be valid, but the parser was unable to parse it,
    /// e.g. `"tomorrow at at 5pm"`
    ParseError,
}

// so that we don't have to change this in both places
// doesn't show up in the docs
pub type NaiveOutput = Result<NaiveDateTime, Error>;

/// Parse an input string into a chrono NaiveDateTime, using the default
/// values from the specified default value where not specified
pub fn parse_with_default_time(input: impl Into<String>, default: NaiveTime) -> NaiveOutput {
    let lexemes = lexer::Lexeme::lex_line(input.into())?;
    let (tree, tokens) = ast::DateTime::parse(lexemes.as_slice()).ok_or(Error::ParseError)?;

    if tokens < lexemes.len() {
        return Err(crate::Error::ParseError);
    };

    let now = Local::now()
        .with_time(default)
        .earliest()
        .ok_or(crate::Error::ParseError)?;

    tree.to_chrono(now).map(|dt| dt.naive_local())
}

/// Parse an input string into a chrono NaiveDateTime, treating the default as
/// if it was the current time.
pub fn parse_relative_to(input: impl Into<String>, default: NaiveDateTime) -> NaiveOutput {
    let lexemes = lexer::Lexeme::lex_line(input.into())?;
    let (tree, tokens) = ast::DateTime::parse(lexemes.as_slice()).ok_or(Error::ParseError)?;

    if tokens < lexemes.len() {
        return Err(crate::Error::ParseError);
    };

    let now = default
        .and_local_timezone(Local)
        .earliest()
        .ok_or(crate::Error::ParseError)?;

    tree.to_chrono(now).map(|dt| dt.naive_local())
}

/// Parse an input string into a chrono NaiveDateTime with the default
/// time being now
pub fn parse(input: impl Into<String>) -> NaiveOutput {
    parse_with_default_time(input, Local::now().time())
}

/// Parse an input string into a chrono DateTime with the given default time. Defaults to None if
/// not given. Time is parsed and returned in the given timezone.
pub fn aware_parse<Tz: TimeZone>(
    input: impl Into<String>,
    relative_to: Option<DateTime<Tz>>,
    tz: Tz,
) -> Result<DateTime<Tz>, Error> {
    let lexemes = lexer::Lexeme::lex_line(input.into())?;
    let (tree, tokens) = ast::DateTime::parse(lexemes.as_slice()).ok_or(Error::ParseError)?;

    if tokens < lexemes.len() {
        return Err(crate::Error::ParseError);
    };

    let now = relative_to.unwrap_or_else(|| tz.from_utc_datetime(&Local::now().naive_utc()));

    tree.to_chrono(now)
}

/// Parse an input string into a chrono DateTime with the given default time. Defaults to None if
/// not given. Time is parsed and returned in the given timezone. Returns all stages of parsing
/// for debugging
#[allow(clippy::type_complexity)]
pub fn debug_parse<Tz: TimeZone>(
    input: impl Into<String>,
    relative_to: Option<DateTime<Tz>>,
    tz: Tz,
) -> (
    Result<Vec<lexer::Lexeme>, Error>,
    Option<(ast::DateTime, usize)>,
    Option<Result<DateTime<Tz>, Error>>,
) {
    let now = relative_to.unwrap_or_else(|| tz.from_utc_datetime(&Local::now().naive_utc()));
    let lexemes_result = lexer::Lexeme::lex_line(input.into());

    if let Ok(lexemes) = &lexemes_result {
        let dt_result = ast::DateTime::parse(lexemes);
        if let Some((dt, _)) = &dt_result {
            let chrono_result = dt.to_chrono(now);
            (lexemes_result, dt_result, Some(chrono_result))
        } else {
            (lexemes_result, dt_result, None)
        }
    } else {
        (lexemes_result, None, None)
    }
}

#[test]
fn test_parse() {
    use chrono::Datelike;
    let input = "2/12/2022";
    let date = parse(input).unwrap();

    assert_eq!(2, date.month());
    assert_eq!(12, date.day());
    assert_eq!(2022, date.year());
}

#[test]
fn test_malformed() {
    let input = "Hello World";
    let date = parse(input);
    assert!(date.is_err());
}

#[test]
fn test_empty() {
    let input = "";
    let date = parse(input);
    assert!(date.is_err());
}
