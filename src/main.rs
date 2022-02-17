use std::io::stdin;
use fuzzydate::parse;

fn main() {
    let mut s = String::new();
    while let Ok(_) = stdin().read_line(&mut s) {
        let date = parse(&s);
        println!("{:?}", date);
        s = String::new();
    }
}
