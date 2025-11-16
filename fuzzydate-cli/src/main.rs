use std::{fmt::Display, process::exit};

use clap::Parser;
use fuzzydate::{parse, parse_relative_to, parse_with_default_time};

#[derive(Parser, Debug)]
struct Args {
    // TODO: handle timezones
    #[arg(short, long, default_value = "%Y-%m-%dT%H:%M:%S")]
    format: String,

    #[arg(short, long, group = "base")]
    default_time: Option<String>,

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

    let result = match args {
        Args {
            default_time: Some(default_time),
            date_string,
            ..
        } => {
            let default_time = unwrap_or_report(parse(default_time));
            unwrap_or_report(parse_with_default_time(
                date_string.join(" "),
                default_time.time(),
            ))
        }
        Args {
            relative_to: Some(relative_to),
            date_string,
            ..
        } => {
            let relative_to = unwrap_or_report(parse(relative_to));
            unwrap_or_report(parse_relative_to(date_string.join(" "), relative_to))
        }
        Args { date_string, .. } => unwrap_or_report(parse(date_string.join(" "))),
    };

    println!("{}", result.format(&args.format));
}
