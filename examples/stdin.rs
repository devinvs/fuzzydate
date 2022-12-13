extern crate fuzzydate;
use fuzzydate::parse;
use std::io::stdin;

fn main() {
    let mut buf = String::new();
    while stdin().read_line(&mut buf).is_ok() {
        let date = parse(&buf);
        println!("{:?}", date);
        buf.clear();
    }
}
