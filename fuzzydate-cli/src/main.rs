use std::{fmt::Display, process::exit, str::FromStr};

use chrono_tz::Tz;
use clap::Parser;
use fuzzydate::aware_parse;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "%+")]
    format: String,

    #[arg(short, long, group = "base")]
    relative_to: Option<String>,

    // TODO: describe default as Local
    #[arg(long, group = "base")]
    input_timezone: Option<String>,

    #[arg(long, group = "base")]
    output_timezone: Option<String>,

    #[arg(default_value = "today")]
    date_string: Vec<String>,
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

    // TODO: fix the expects here
    // TODO: fix the timezones
    let input_tz_str = args
        .input_timezone
        .unwrap_or_else(|| unwrap_or_report(iana_time_zone::get_timezone()));
    let input_tz = unwrap_or_report(Tz::from_str(&input_tz_str));

    let relative_to = args
        .relative_to
        .map(|dt| aware_parse(dt, None, input_tz).expect("Invalid datetime"));

    let result = unwrap_or_report(aware_parse(fuzzy_str, relative_to, input_tz));

    let output_tz_str = args
        .output_timezone
        .unwrap_or_else(|| unwrap_or_report(iana_time_zone::get_timezone()));
    let output_tz = unwrap_or_report(Tz::from_str(&output_tz_str));

    let result = result.with_timezone(&output_tz);

    println!("{}", result.format(&args.format));
}
