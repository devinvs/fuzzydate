use clap::Parser;
use fuzzydate::{parse, parse_relative_to, parse_with_default_time};

#[derive(Parser, Debug)]
struct Args {
    date_string: String,

    // TODO: handle timezones
    #[arg(default_value = "%Y-%m-%dT%H:%M:%S")]
    format: String,

    #[arg(short, long, group = "base")]
    default_time: Option<String>,

    #[arg(short, long, group = "base")]
    relative_to: Option<String>,
}

fn main() {
    let args = Args::parse();

    let result = match args {
        // FIXME: report errors instead of unwrapping
        Args {
            default_time: Some(default_time),
            date_string,
            ..
        } => {
            let default_time = parse(default_time).unwrap();
            parse_with_default_time(date_string, default_time.time())
        }
        Args {
            relative_to: Some(relative_to),
            date_string,
            ..
        } => {
            let relative_to = parse(relative_to).unwrap();
            parse_relative_to(date_string, relative_to)
        }
        Args { date_string, .. } => parse(date_string),
    };

    println!("{}", result.unwrap().format(&args.format));
}
