use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    /// Hashmap of keywords to the lexeme that they represent
    /// Used as definitive source during lexeme
    static ref KEYWORDS: HashMap<&'static str, Lexeme> = {
        let mut map = HashMap::new();

        map.insert("on", Lexeme::On);
        map.insert("at", Lexeme::At);
        map.insert("an", Lexeme::An);
        map.insert("after", Lexeme::After);
        map.insert("last", Lexeme::Last);
        map.insert("this", Lexeme::This);
        map.insert("next", Lexeme::Next);
        map.insert("monday", Lexeme::Monday);
        map.insert("tuesday", Lexeme::Tuesday);
        map.insert("wednesday", Lexeme::Wednesday);
        map.insert("thursday", Lexeme::Thursday);
        map.insert("friday", Lexeme::Friday);
        map.insert("saturday", Lexeme::Saturday);
        map.insert("sunday", Lexeme::Sunday);
        map.insert("january", Lexeme::January);
        map.insert("february", Lexeme::February);
        map.insert("march", Lexeme::March);
        map.insert("april", Lexeme::April);
        map.insert("may", Lexeme::May);
        map.insert("june", Lexeme::June);
        map.insert("july", Lexeme::July);
        map.insert("august", Lexeme::August);
        map.insert("september", Lexeme::September);
        map.insert("october", Lexeme::October);
        map.insert("november", Lexeme::November);
        map.insert("december", Lexeme::December);
        map.insert("jan", Lexeme::January);
        map.insert("feb", Lexeme::February);
        map.insert("mar", Lexeme::March);
        map.insert("apr", Lexeme::April);
        map.insert("jun", Lexeme::June);
        map.insert("jul", Lexeme::July);
        map.insert("aug", Lexeme::August);
        map.insert("sep", Lexeme::September);
        map.insert("oct", Lexeme::October);
        map.insert("nov", Lexeme::November);
        map.insert("dec", Lexeme::December);
        map.insert("am", Lexeme::AM);
        map.insert("pm", Lexeme::PM);
        map.insert("day", Lexeme::Day);
        map.insert("days", Lexeme::Day);
        map.insert("week", Lexeme::Week);
        map.insert("weeks", Lexeme::Week);
        map.insert("month", Lexeme::Month);
        map.insert("months", Lexeme::Month);
        map.insert("year", Lexeme::Year);
        map.insert("years", Lexeme::Year);
        map.insert("hour", Lexeme::Hour);
        map.insert("hours", Lexeme::Hour);
        map.insert("min", Lexeme::Minute);
        map.insert("mins", Lexeme::Minute);
        map.insert("minute", Lexeme::Minute);
        map.insert("minutes", Lexeme::Minute);
        map.insert("and", Lexeme::And);
        map.insert("today", Lexeme::Today);
        map.insert("tomorrow", Lexeme::Tomorrow);
        map.insert("yesterday", Lexeme::Yesterday);
        map.insert("now", Lexeme::Now);
        map.insert("from", Lexeme::From);
        map.insert("zero", Lexeme::Zero);
        map.insert("one", Lexeme::One);
        map.insert("two", Lexeme::Two);
        map.insert("three", Lexeme::Three);
        map.insert("four", Lexeme::Four);
        map.insert("five", Lexeme::Five);
        map.insert("six", Lexeme::Six);
        map.insert("seven", Lexeme::Seven);
        map.insert("eight", Lexeme::Eight);
        map.insert("nine", Lexeme::Nine);
        map.insert("ten", Lexeme::Ten);
        map.insert("eleven", Lexeme::Eleven);
        map.insert("twelve", Lexeme::Twelve);
        map.insert("thirteen", Lexeme::Thirteen);
        map.insert("fourteen", Lexeme::Fourteen);
        map.insert("fifteen", Lexeme::Fifteen);
        map.insert("sixteen", Lexeme::Sixteen);
        map.insert("seventeen", Lexeme::Seventeen);
        map.insert("eighteen", Lexeme::Eighteen);
        map.insert("nineteen", Lexeme::Nineteen);
        map.insert("twenty", Lexeme::Twenty);
        map.insert("thirty", Lexeme::Thirty);
        map.insert("fourty", Lexeme::Fourty);
        map.insert("fifty", Lexeme::Fifty);
        map.insert("sixty", Lexeme::Sixty);
        map.insert("seventy", Lexeme::Seventy);
        map.insert("eighty", Lexeme::Eighty);
        map.insert("ninety", Lexeme::Ninety);
        map.insert("hundred", Lexeme::Hundred);
        map.insert("thousand", Lexeme::Thousand);
        map.insert("million", Lexeme::Million);
        map.insert("billion", Lexeme::Billion);
        map.insert("before", Lexeme::Before);
        map.insert("ago", Lexeme::Ago);
        map.insert("midnight", Lexeme::Midnight);
        map.insert("noon", Lexeme::Noon);
        map.insert("a", Lexeme::A);
        map.insert("the", Lexeme::The);

        map
    };
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
/// Enum for all valid tokens in the parse string
pub enum Lexeme {
    A,
    An,
    On,
    At,
    The,
    Dash,
    Today,
    Tomorrow,
    Yesterday,
    From,
    Now,
    And,
    Comma,
    Colon,
    Dot,
    After,
    Num(u32),
    This,
    Next,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
    AM,
    PM,
    Day,
    Week,
    Hour,
    Minute,
    Month,
    Year,
    Slash,
    Before,
    Ago,
    Midnight,
    Noon,

    // Number parsing lexemes
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Eleven,
    Twelve,
    Thirteen,
    Fourteen,
    Fifteen,
    Sixteen,
    Seventeen,
    Eighteen,
    Nineteen,
    Twenty,
    Thirty,
    Fourty,
    Fifty,
    Sixty,
    Seventy,
    Eighty,
    Ninety,
    Hundred,
    Thousand,
    Million,
    Billion,
    Last,
}

const LEXER_STACK_SIZE: usize = 20;

impl Lexeme {
    /// Lex a string into a list of Lexemes
    pub fn lex_line(s: String) -> Result<Vec<Lexeme>, crate::Error> {
        // Convert s to lowercase to remove case sensitive behaviour
        let s = s.to_lowercase();

        let mut lexemes = Vec::new(); // List of Lexemes
        let chars = s.chars(); // Character iterator
        let mut stack = String::with_capacity(LEXER_STACK_SIZE);

        // Convenience closure which takes a reference to our stack
        // and our lexemes, searches our keyword map for the stack,
        // tries to convert the stack into a integer, adds the appropriate
        // lexemes if successfully, and zeroes out the stack
        let push_lexeme = |stack: &mut String, ls: &mut Vec<Lexeme>| {
            if stack.is_empty() {
                Ok(())
            } else if let Some(l) = KEYWORDS.get(stack.as_str()) {
                ls.push(*l);
                *stack = String::with_capacity(10);
                Ok(())
            } else if let Ok(num) = stack.parse::<u32>() {
                ls.push(Lexeme::Num(num));
                stack.clear();
                Ok(())
            } else {
                Err(crate::Error::UnrecognizedToken(stack.clone()))
            }
        };

        // While we have characters left in the string
        for c in chars {
            // Whitespace always separates lexemes, push whatever we have
            // on the stack and continue to the next character
            if c.is_whitespace() {
                push_lexeme(&mut stack, &mut lexemes)?;
                continue;
            }

            if stack
                .chars()
                .last()
                // switching from a digit to alpha or alpha to digit is the end of a lexeme
                .is_some_and(|sc| sc.is_ascii_digit() != c.is_ascii_digit())
            {
                push_lexeme(&mut stack, &mut lexemes)?;
            }

            if stack.len() == LEXER_STACK_SIZE {
                return Err(crate::Error::ParseError);
            }

            match c {
                // Comma separates lexemes, push stack and add comma
                ',' => {
                    push_lexeme(&mut stack, &mut lexemes)?;
                    lexemes.push(Lexeme::Comma);
                }
                // Colon separates lexemes, push stack and add colon
                ':' => {
                    push_lexeme(&mut stack, &mut lexemes)?;
                    lexemes.push(Lexeme::Colon);
                }
                // Slash separates lexemes, push stack and add slash
                '/' => {
                    push_lexeme(&mut stack, &mut lexemes)?;
                    lexemes.push(Lexeme::Slash);
                }
                // Dash separates lexemes, push stack and add dash
                '-' => {
                    push_lexeme(&mut stack, &mut lexemes)?;
                    lexemes.push(Lexeme::Dash);
                }
                // Dot separates lexemes, push stack and add dash
                '.' => {
                    push_lexeme(&mut stack, &mut lexemes)?;
                    lexemes.push(Lexeme::Dot);
                }
                // Else just add the character to our stack
                _ => stack.push(c),
            }
        }

        // If any characters remaining on our stack, push them
        push_lexeme(&mut stack, &mut lexemes)?;

        Ok(lexemes)
    }
}

#[test]
fn test_simple_date() {
    let input = "5/2/2022".to_string();
    assert_eq!(
        Ok(vec![
            Lexeme::Num(5),
            Lexeme::Slash,
            Lexeme::Num(2),
            Lexeme::Slash,
            Lexeme::Num(2022)
        ]),
        Lexeme::lex_line(input)
    );
}

#[test]
fn test_complex_relative_date_time() {
    let input = "fifty-five days from january 1, 2010 5:00".to_string();
    assert_eq!(
        Ok(vec![
            Lexeme::Fifty,
            Lexeme::Dash,
            Lexeme::Five,
            Lexeme::Day,
            Lexeme::From,
            Lexeme::January,
            Lexeme::Num(1),
            Lexeme::Comma,
            Lexeme::Num(2010),
            Lexeme::Num(5),
            Lexeme::Colon,
            Lexeme::Num(0)
        ]),
        Lexeme::lex_line(input)
    );
}

#[test]
fn test_unknown_token() {
    let input = "Hello World".to_string();
    assert!(Lexeme::lex_line(input).is_err());
}

#[test]
fn test_am_without_space() {
    let input = "10am".to_string();
    assert_eq!(
        Ok(vec![Lexeme::Num(10), Lexeme::AM,]),
        Lexeme::lex_line(input)
    );
}
