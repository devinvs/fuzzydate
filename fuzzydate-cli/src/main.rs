use std::{fmt::Display, process::exit, str::FromStr};

use chrono::FixedOffset;
use chrono_tz::Tz;
use clap::Parser;
use fuzzydate::{aware_parse, debug_parse};

#[derive(Parser, Debug)]
struct Args {
    /// Output format for datetime string. See https://docs.rs/chrono/latest/chrono/format/strftime/index.html for supported options
    #[arg(short, long, default_value = "%Y-%m-%dT%H:%M:%S%:z")]
    format: String,

    /// Report the parsing steps for this expression
    #[arg(long)]
    debug: bool,

    /// Instant in time that should be considered the current time, formatted as an RFC3339 string
    #[arg(short, long, group = "base", value_parser = parse_datetime)]
    relative_to: Option<chrono::DateTime<FixedOffset>>,

    /// Timezone for inferred and relative values. Defaults to system timezone
    #[arg(long, group = "base")]
    input_timezone: Option<String>,

    /// Timezone to convert output value to. Defaults to system timezone
    #[arg(long, group = "base")]
    output_timezone: Option<String>,

    #[arg(default_value = "today")]
    date_string: Vec<String>,
}

fn parse_datetime(str: &str) -> Result<chrono::DateTime<FixedOffset>, chrono::ParseError> {
    chrono::DateTime::parse_from_rfc3339(str)
}

fn unwrap_or_report<T, E: Display>(arg: Result<T, E>) -> T {
    match arg {
        Ok(value) => value,
        Err(msg) => {
            eprintln!("{}", msg);
            exit(1);
        }
    }
}

fn main() {
    let args = Args::parse();

    let fuzzy_str = args.date_string.join(" ");

    let input_tz_str = args
        .input_timezone
        .unwrap_or_else(|| unwrap_or_report(iana_time_zone::get_timezone()));
    let input_tz = unwrap_or_report(Tz::from_str(&input_tz_str));

    let relative_to = args.relative_to.map(|dt| dt.with_timezone(&input_tz));

    if args.debug {
        let (lexemes, ast, result) = debug_parse(&fuzzy_str, relative_to, input_tz);

        let _ = dbg!(lexemes);
        dbg!(ast);
        dbg!(result);
    }

    let result = unwrap_or_report(aware_parse(fuzzy_str, relative_to, input_tz));

    let output_tz_str = args
        .output_timezone
        .unwrap_or_else(|| unwrap_or_report(iana_time_zone::get_timezone()));
    let output_tz = unwrap_or_report(Tz::from_str(&output_tz_str));

    let result = result.with_timezone(&output_tz);

    println!("{}", result.format(&args.format));
}
