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
//! ; TODO: in <num> <duration>
//! <datetime> ::= <time>
//!              | <time> , <date_expr>
//!              | <time> <date_expr>
//!              | <date_expr>
//!              | <date_expr> <time>
//!              | <date_expr> , <time>
//!              | <date_expr> at <time>
//!              | <duration> after <datetime>
//!              | <duration> from <datetime>
//!              | <duration> before <datetime>
//!              | <duration> ago
//!              | now
//!
//! <date_expr> ::= <date>
//!               | <date> <relative_specifier> <unit>
//!               | <duration> ago              ; duration must be for a whole number of days
//!               | <duration> after <date>
//!               | <duration> from <date>
//!               | <duration> before <date>
//!               | <relative_specifier> <weekday>
//!               | <relative_specifier> <unit>
//!               | <weekday>
//!
//! <date> ::= today | tomorrow | yesterday  ; whole-day offsets -1 / 0 / +1 from the anchor date
//!          | <num> / <num> / <num>  ; if the first value is > 1000, format of triples is (year, month, day)
//!          | <num> - <num> - <num>  ; M[M] - D[D] - Y[Y][YY]
//!          | <num> . <num> . <num>  ; D[D] . M[M] . Y[Y][YY]
//!          | <month> <num> <num>
//!          | <num> <month> <num>
//!          | <month> <num>
//!          | <num> <month>
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
//! <duration> ::= <num> <uni>
//!              | <article> <unit>
//!              | <duration> and <duration>
//!
//! <article> ::= a
//!             | an
//!             | the
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

/// Inclusive lower bound for `Lexeme::DayOffset` values accepted by the date parser (~±100 years).
pub const DAY_OFFSET_SANITY_MIN: i32 = -36_525;
/// Inclusive upper bound for `Lexeme::DayOffset` values accepted by the date parser (~±100 years).
pub const DAY_OFFSET_SANITY_MAX: i32 = 36_525;

/// Whether `days` is within [`DAY_OFFSET_SANITY_MIN`]..=[`DAY_OFFSET_SANITY_MAX`].
#[inline]
pub fn day_offset_in_sanity_range(days: i32) -> bool {
    (DAY_OFFSET_SANITY_MIN..=DAY_OFFSET_SANITY_MAX).contains(&days)
}

mod ast;
mod lexer;

use chrono::{DateTime, Local, NaiveDateTime, NaiveTime, TimeZone};
use std::sync::Arc;

pub use lexer::{Lexeme, Lexicon};

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

/// Configurable fuzzy date parser: holds a [`Lexicon`] for keyword recognition.
#[derive(Clone, Debug)]
pub struct Parser {
    lexicon: Arc<Lexicon>,
}

impl Parser {
    pub fn new(lexicon: Lexicon) -> Self {
        Self {
            lexicon: Arc::new(lexicon),
        }
    }

    pub fn english() -> Self {
        Self {
            lexicon: lexer::shared_english_lexicon(),
        }
    }

    pub fn lexicon(&self) -> &Lexicon {
        self.lexicon.as_ref()
    }

    pub fn into_lexicon(self) -> Lexicon {
        Arc::try_unwrap(self.lexicon).unwrap_or_else(|arc| (*arc).clone())
    }

    pub fn lex_line(&self, input: impl Into<String>) -> Result<Vec<Lexeme>, Error> {
        Lexeme::lex_line_with_lexicon(input.into(), self.lexicon.as_ref())
    }

    /// Parse an input string into a chrono NaiveDateTime, using the default
    /// values from the specified default value where not specified
    #[deprecated = "superseded by parse_relative_to"]
    pub fn parse_with_default_time(
        &self,
        input: impl Into<String>,
        default: NaiveTime,
    ) -> NaiveOutput {
        let lexemes = self.lex_line(input.into())?;
        let (tree, tokens) = ast::DateTime::parse(lexemes.as_slice()).ok_or(Error::ParseError)?;

        if tokens < lexemes.len() {
            return Err(crate::Error::ParseError);
        }

        let now = Local::now()
            .with_time(default)
            .earliest()
            .ok_or(crate::Error::ParseError)?;

        tree.to_chrono(now).map(|dt| dt.naive_local())
    }

    pub fn parse_relative_to(
        &self,
        input: impl Into<String>,
        relative_to: NaiveDateTime,
    ) -> NaiveOutput {
        let lexemes = self.lex_line(input.into())?;
        let (tree, tokens) = ast::DateTime::parse(lexemes.as_slice()).ok_or(Error::ParseError)?;

        if tokens < lexemes.len() {
            return Err(crate::Error::ParseError);
        }

        let now = relative_to
            .and_local_timezone(Local)
            .earliest()
            .ok_or(crate::Error::ParseError)?;

        tree.to_chrono(now).map(|dt| dt.naive_local())
    }

    pub fn parse(&self, input: impl Into<String>) -> NaiveOutput {
        self.parse_relative_to(input, Local::now().naive_local())
    }

    pub fn aware_parse<Tz: TimeZone>(
        &self,
        input: impl Into<String>,
        relative_to: Option<DateTime<Tz>>,
        tz: Tz,
    ) -> Result<DateTime<Tz>, Error> {
        let lexemes = self.lex_line(input.into())?;
        let (tree, tokens) = ast::DateTime::parse(lexemes.as_slice()).ok_or(Error::ParseError)?;

        if tokens < lexemes.len() {
            return Err(crate::Error::ParseError);
        }

        let now = relative_to
            .unwrap_or_else(|| tz.from_utc_datetime(&Local::now().naive_utc()))
            .with_timezone(&tz);

        tree.to_chrono(now)
    }

    #[allow(clippy::type_complexity)]
    pub fn debug_parse<Tz: TimeZone>(
        &self,
        input: impl Into<String>,
        relative_to: Option<DateTime<Tz>>,
        tz: Tz,
    ) -> (
        Result<Vec<Lexeme>, Error>,
        Option<(ast::DateTime, usize)>,
        Option<Result<DateTime<Tz>, Error>>,
    ) {
        let now = relative_to.unwrap_or_else(|| tz.from_utc_datetime(&Local::now().naive_utc()));
        let lexemes_result = self.lex_line(input.into());

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
}

/// Parse an input string into a chrono NaiveDateTime, using the default
/// values from the specified default value where not specified
#[deprecated = "superseded by parse_relative_to"]
#[allow(deprecated)]
pub fn parse_with_default_time(input: impl Into<String>, default: NaiveTime) -> NaiveOutput {
    Parser::english().parse_with_default_time(input, default)
}

/// Parse an input string into a chrono NaiveDateTime, using relative_to as the current time.
pub fn parse_relative_to(input: impl Into<String>, relative_to: NaiveDateTime) -> NaiveOutput {
    Parser::english().parse_relative_to(input, relative_to)
}

/// Parse an input string into a chrono NaiveDateTime using the system local time for any missing values.
pub fn parse(input: impl Into<String>) -> NaiveOutput {
    Parser::english().parse(input)
}

/// Parse an input string into a timezone-aware chrono DateTime using the given time and timezone.
pub fn aware_parse<Tz: TimeZone>(
    input: impl Into<String>,
    relative_to: Option<DateTime<Tz>>,
    tz: Tz,
) -> Result<DateTime<Tz>, Error> {
    Parser::english().aware_parse(input, relative_to, tz)
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
    Parser::english().debug_parse(input, relative_to, tz)
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

#[test]
fn test_parse_today_yesterday_tomorrow_dates() {
    use chrono::{Duration, NaiveDate};

    let anchor = NaiveDate::from_ymd_opt(2026, 5, 15)
        .unwrap()
        .and_hms_opt(14, 30, 45)
        .unwrap();
    let d = anchor.date();

    let today = parse_relative_to("today", anchor).unwrap();
    assert_eq!(today.date(), d);
    assert_eq!(today.time(), anchor.time());

    assert_eq!(
        parse_relative_to("yesterday", anchor).unwrap().date(),
        d.checked_sub_signed(Duration::days(1)).unwrap()
    );
    assert_eq!(
        parse_relative_to("tomorrow", anchor).unwrap().date(),
        d.checked_add_signed(Duration::days(1)).unwrap()
    );
}

#[test]
fn test_parse_yesterday_across_leap_day() {
    use chrono::NaiveDate;

    let march_first_leap = NaiveDate::from_ymd_opt(2024, 3, 1)
        .unwrap()
        .and_hms_opt(9, 0, 0)
        .unwrap();
    assert_eq!(
        parse_relative_to("yesterday", march_first_leap)
            .unwrap()
            .date(),
        NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()
    );
}

#[test]
fn test_parser_merged_lexicon_overmorgen_eergisteren() {
    use chrono::{Duration, NaiveDate};

    let p = Parser::new(Lexicon::english().merged_with(&Lexicon::dutch()));
    let anchor = NaiveDate::from_ymd_opt(2026, 6, 1)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let d = anchor.date();
    assert_eq!(
        p.parse_relative_to("overmorgen", anchor).unwrap().date(),
        d.checked_add_signed(Duration::days(2)).unwrap()
    );
    assert_eq!(
        p.parse_relative_to("eergisteren", anchor).unwrap().date(),
        d.checked_sub_signed(Duration::days(2)).unwrap()
    );
    assert_eq!(p.parse_relative_to("today", anchor).unwrap().date(), d);
}

#[test]
fn test_dutch_only_lexicon_no_english_today() {
    use chrono::NaiveDate;

    let p = Parser::new(Lexicon::dutch());
    let anchor = NaiveDate::from_ymd_opt(2026, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    assert!(p.parse_relative_to("today", anchor).is_err());
    assert_eq!(
        p.parse_relative_to("vandaag", anchor).unwrap().date(),
        anchor.date()
    );
}
