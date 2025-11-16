use std::{fmt::Display, process::exit, str::FromStr};

use chrono::Local;
use chrono_tz::Tz;
use clap::Parser;
use fuzzydate::{aware_parse, parse, parse_relative_to, parse_with_default_time};

#[derive(Parser, Debug)]
struct Args {
    // TODO: handle timezones
    #[arg(short, long, default_value = "%Y-%m-%dT%H:%M:%S%Z")]
    format: String,

    #[arg(short, long, group = "base")]
    relative_to: Option<String>,

    #[arg(long, group = "base")]
    input_timezone: Option<String>,

    #[arg(long, group = "base")]
    // TODO: default to local
    output_timezone: Option<String>,

    #[arg(short, long, group = "base")]
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
    let input_tz = args
        .input_timezone
        .map_or(Local, |tz| Tz::from_str(&tz).expect("Invalid Timezone"));
    let output_tz = args
        .output_timezone
        .map_or(Local, |tz| Tz::from_str(&tz).expect("Invalid timezone"));

    let relative_to = args
        .relative_to
        .map(|dt| aware_parse(dt, None, input_tz).expect("Invalid datetime"));

    let result =
        aware_parse(args.date_string.join(" "), relative_to, input_tz).expect("Parse Error");

    println!("{}", result.format(&args.format));
}
