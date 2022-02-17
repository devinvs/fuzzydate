extern crate fuzzydate;
use fuzzydate::parse;
use std::io::stdin;

fn main() {
    let mut buf = String::new();
    while let Ok(_) =  stdin().read_line(&mut buf) {
        let date = parse(&buf);
        println!("{:?}", date);
        buf.clear();
    }
}
