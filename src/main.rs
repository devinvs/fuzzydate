use std::io::stdin;

use fuzzydate::{lexer::Lexeme, ast::DateTime};

fn main() {
    let mut s = String::new();
    while let Ok(_) = stdin().read_line(&mut s) {
        let lexemes = Lexeme::lex_line(s.clone()).unwrap();
        let tree = DateTime::parse(&mut lexemes.as_slice()).unwrap();
        let date = tree.map(|t| t.0.to_chrono());
        println!("{:?}", date);
        s = String::new();
    }
}
