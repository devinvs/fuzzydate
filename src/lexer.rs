use lazy_static::lazy_static;
use std::collections::HashMap;
use std::collections::HashSet;

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, Lexeme> = {
        let mut map = HashMap::new();

        map.insert("an", Lexeme::An);
        map.insert("after", Lexeme::After);
        map.insert("last", Lexeme::Last);
        map.insert("random", Lexeme::Random);
        map.insert("between", Lexeme::Between);
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
        map.insert("a", Lexeme::A);
        map.insert("the", Lexeme::The);

        map
    };
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Lexeme {
    A,
    An,
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
    After,
    Random,
    Between,
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
    Last
}

impl Lexeme {
    pub fn lex_line(s: String) -> Result<Vec<Lexeme>, String> {
        let s = s.to_lowercase();
        let mut lexemes = Vec::new();
        let mut chars = s.chars();
        let mut stack = String::with_capacity(10);

        let push_lexeme = |stack: &mut String, ls: &mut Vec<Lexeme>| -> Result<(), String> {
            if stack.is_empty() {
                Ok(())
            } else if let Some(l) = KEYWORDS.get(stack.as_str()) {
                ls.push(l.clone());
                *stack = String::with_capacity(10);
                Ok(())
            } else if let Ok(num) = stack.parse::<u32>() {
                ls.push(Lexeme::Num(num));
                *stack = String::with_capacity(10);
                Ok(())
            } else {
                Err("".into())
            }
        };

        while let Some(c) = chars.next() {
            if c.is_whitespace() {
                push_lexeme(&mut stack, &mut lexemes)?;
                continue;
            }

            match c {
                ',' => {
                    push_lexeme(&mut stack, &mut lexemes)?;
                    lexemes.push(Lexeme::Comma);
                }
                ':' => {
                    push_lexeme(&mut stack, &mut lexemes)?;
                    lexemes.push(Lexeme::Colon);
                }
                '/' => {
                    push_lexeme(&mut stack, &mut lexemes)?;
                    lexemes.push(Lexeme::Slash);
                }
                '-' => {
                    push_lexeme(&mut stack, &mut lexemes)?;
                    lexemes.push(Lexeme::Dash);
                }
                _ => stack.push(c)
            }
        }

        push_lexeme(&mut stack, &mut lexemes)?;

        Ok(lexemes)
    }
}
