pub mod lexer;
pub mod ast;

#[test]
fn test_parse() {
    let input = "2/16/2022";
    let lexemes = lexer::Lexeme::lex_line(input.into()).unwrap();
    println!("{:?}", lexemes);
    let tree = ast::DateTime::parse(&mut lexemes.as_slice());

    assert_eq!(tree,
        Ok(Some(ast::DateTime::DateTime(
            ast::Date::MonthNumDayYear(2,16,2022),
            ast::Time::Empty
    ))));
}
