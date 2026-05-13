use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

fn insert_english_keywords(map: &mut HashMap<String, Lexeme>) {
    map.insert("on".into(), Lexeme::On);
    map.insert("at".into(), Lexeme::At);
    map.insert("an".into(), Lexeme::An);
    map.insert("in".into(), Lexeme::In);
    map.insert("after".into(), Lexeme::After);
    map.insert("last".into(), Lexeme::Last);
    map.insert("this".into(), Lexeme::This);
    map.insert("next".into(), Lexeme::Next);
    map.insert("monday".into(), Lexeme::Monday);
    map.insert("tuesday".into(), Lexeme::Tuesday);
    map.insert("wednesday".into(), Lexeme::Wednesday);
    map.insert("thursday".into(), Lexeme::Thursday);
    map.insert("friday".into(), Lexeme::Friday);
    map.insert("saturday".into(), Lexeme::Saturday);
    map.insert("sunday".into(), Lexeme::Sunday);
    map.insert("january".into(), Lexeme::January);
    map.insert("february".into(), Lexeme::February);
    map.insert("march".into(), Lexeme::March);
    map.insert("april".into(), Lexeme::April);
    map.insert("may".into(), Lexeme::May);
    map.insert("june".into(), Lexeme::June);
    map.insert("july".into(), Lexeme::July);
    map.insert("august".into(), Lexeme::August);
    map.insert("september".into(), Lexeme::September);
    map.insert("october".into(), Lexeme::October);
    map.insert("november".into(), Lexeme::November);
    map.insert("december".into(), Lexeme::December);
    map.insert("jan".into(), Lexeme::January);
    map.insert("feb".into(), Lexeme::February);
    map.insert("mar".into(), Lexeme::March);
    map.insert("apr".into(), Lexeme::April);
    map.insert("jun".into(), Lexeme::June);
    map.insert("jul".into(), Lexeme::July);
    map.insert("aug".into(), Lexeme::August);
    map.insert("sep".into(), Lexeme::September);
    map.insert("oct".into(), Lexeme::October);
    map.insert("nov".into(), Lexeme::November);
    map.insert("dec".into(), Lexeme::December);
    map.insert("am".into(), Lexeme::AM);
    map.insert("pm".into(), Lexeme::PM);
    map.insert("day".into(), Lexeme::Day);
    map.insert("days".into(), Lexeme::Day);
    map.insert("week".into(), Lexeme::Week);
    map.insert("weeks".into(), Lexeme::Week);
    map.insert("month".into(), Lexeme::Month);
    map.insert("months".into(), Lexeme::Month);
    map.insert("year".into(), Lexeme::Year);
    map.insert("years".into(), Lexeme::Year);
    map.insert("hour".into(), Lexeme::Hour);
    map.insert("hours".into(), Lexeme::Hour);
    map.insert("min".into(), Lexeme::Minute);
    map.insert("mins".into(), Lexeme::Minute);
    map.insert("minute".into(), Lexeme::Minute);
    map.insert("minutes".into(), Lexeme::Minute);
    map.insert("and".into(), Lexeme::And);
    map.insert("today".into(), Lexeme::DayOffset(0));
    map.insert("tomorrow".into(), Lexeme::DayOffset(1));
    map.insert("yesterday".into(), Lexeme::DayOffset(-1));
    map.insert("now".into(), Lexeme::Now);
    map.insert("from".into(), Lexeme::From);
    map.insert("zero".into(), Lexeme::Zero);
    map.insert("one".into(), Lexeme::One);
    map.insert("two".into(), Lexeme::Two);
    map.insert("three".into(), Lexeme::Three);
    map.insert("four".into(), Lexeme::Four);
    map.insert("five".into(), Lexeme::Five);
    map.insert("six".into(), Lexeme::Six);
    map.insert("seven".into(), Lexeme::Seven);
    map.insert("eight".into(), Lexeme::Eight);
    map.insert("nine".into(), Lexeme::Nine);
    map.insert("ten".into(), Lexeme::Ten);
    map.insert("eleven".into(), Lexeme::Eleven);
    map.insert("twelve".into(), Lexeme::Twelve);
    map.insert("thirteen".into(), Lexeme::Thirteen);
    map.insert("fourteen".into(), Lexeme::Fourteen);
    map.insert("fifteen".into(), Lexeme::Fifteen);
    map.insert("sixteen".into(), Lexeme::Sixteen);
    map.insert("seventeen".into(), Lexeme::Seventeen);
    map.insert("eighteen".into(), Lexeme::Eighteen);
    map.insert("nineteen".into(), Lexeme::Nineteen);
    map.insert("twenty".into(), Lexeme::Twenty);
    map.insert("thirty".into(), Lexeme::Thirty);
    map.insert("fourty".into(), Lexeme::Fourty);
    map.insert("fifty".into(), Lexeme::Fifty);
    map.insert("sixty".into(), Lexeme::Sixty);
    map.insert("seventy".into(), Lexeme::Seventy);
    map.insert("eighty".into(), Lexeme::Eighty);
    map.insert("ninety".into(), Lexeme::Ninety);
    map.insert("hundred".into(), Lexeme::Hundred);
    map.insert("thousand".into(), Lexeme::Thousand);
    map.insert("million".into(), Lexeme::Million);
    map.insert("billion".into(), Lexeme::Billion);
    map.insert("first".into(), Lexeme::One);
    map.insert("second".into(), Lexeme::Two);
    map.insert("third".into(), Lexeme::Three);
    map.insert("fourth".into(), Lexeme::Four);
    map.insert("fifth".into(), Lexeme::Five);
    map.insert("sixth".into(), Lexeme::Six);
    map.insert("seventh".into(), Lexeme::Seven);
    map.insert("eigth".into(), Lexeme::Eight);
    map.insert("ninth".into(), Lexeme::Nine);
    map.insert("tenth".into(), Lexeme::Ten);
    map.insert("eleventh".into(), Lexeme::Eleven);
    map.insert("twelfth".into(), Lexeme::Twelve);
    map.insert("thirteenth".into(), Lexeme::Thirteen);
    map.insert("fourteenth".into(), Lexeme::Fourteen);
    map.insert("fifteenth".into(), Lexeme::Fifteen);
    map.insert("sixteenth".into(), Lexeme::Sixteen);
    map.insert("seventeenth".into(), Lexeme::Seventeen);
    map.insert("eighteenth".into(), Lexeme::Eighteen);
    map.insert("nineteenth".into(), Lexeme::Nineteen);
    map.insert("twentieth".into(), Lexeme::Twenty);
    map.insert("thirtieth".into(), Lexeme::Thirty);
    map.insert("fourtieth".into(), Lexeme::Fourty);
    map.insert("fiftieth".into(), Lexeme::Fifty);
    map.insert("sixtieth".into(), Lexeme::Sixty);
    map.insert("seventieth".into(), Lexeme::Seventy);
    map.insert("eightieth".into(), Lexeme::Eighty);
    map.insert("ninetieth".into(), Lexeme::Ninety);
    map.insert("hundredth".into(), Lexeme::Hundred);
    map.insert("thousandth".into(), Lexeme::Thousand);
    map.insert("millionth".into(), Lexeme::Million);
    map.insert("billionth".into(), Lexeme::Billion);
    map.insert("before".into(), Lexeme::Before);
    map.insert("ago".into(), Lexeme::Ago);
    map.insert("midnight".into(), Lexeme::Midnight);
    map.insert("noon".into(), Lexeme::Noon);
    map.insert("a".into(), Lexeme::A);
    map.insert("the".into(), Lexeme::The);
    map.insert("nd".into(), Lexeme::ND);
    map.insert("st".into(), Lexeme::ST);
    map.insert("rd".into(), Lexeme::RD);
    map.insert("th".into(), Lexeme::RD);
}

/// Dutch keywords for calendar-day relatives. Combine with [`Lexicon::english`] (or another base)
/// via [`Lexicon::merged_with`] for bilingual input; use alone for Dutch-only relative days.
fn insert_dutch_keywords(map: &mut HashMap<String, Lexeme>) {
    map.insert("vandaag".into(), Lexeme::DayOffset(0));
    map.insert("morgen".into(), Lexeme::DayOffset(1));
    map.insert("gisteren".into(), Lexeme::DayOffset(-1));
    map.insert("overmorgen".into(), Lexeme::DayOffset(2));
    map.insert("morgenavond".into(), Lexeme::DayOffset(1));
    map.insert("jaar".into(), Lexeme::Year);
    map.insert("volgend".into(), Lexeme::Next);
    map.insert("volgende".into(), Lexeme::Next);
    map.insert("vorige".into(), Lexeme::Last);
    map.insert("vorig".into(), Lexeme::Last);
    map.insert("eergisteren".into(), Lexeme::DayOffset(-2));
    map.insert("eerste".into(), Lexeme::One);
    map.insert("tweede".into(), Lexeme::Two);
    map.insert("derde".into(), Lexeme::Three);
    map.insert("vierde".into(), Lexeme::Four);
    map.insert("vijfde".into(), Lexeme::Five);
    map.insert("zesde".into(), Lexeme::Six);
    map.insert("zevende".into(), Lexeme::Seven);
}

/// Keyword table for [`Lexeme::lex_line_with_lexicon`].
#[derive(Clone, Debug, Default)]
pub struct Lexicon {
    keywords: HashMap<String, Lexeme>,
}

impl Lexicon {
    pub fn empty() -> Self {
        Self {
            keywords: HashMap::new(),
        }
    }

    /// Full English keyword set (previous default lexer behaviour).
    pub fn english() -> Self {
        let mut keywords = HashMap::new();
        insert_english_keywords(&mut keywords);
        Self { keywords }
    }

    /// Minimal Dutch set: day-offset words including `overmorgen` / `eergisteren`.
    /// Does not include English; merge with [`Lexicon::english`] if you need both.
    pub fn dutch() -> Self {
        let mut keywords = HashMap::new();
        insert_dutch_keywords(&mut keywords);
        Self { keywords }
    }

    /// Insert or overwrite a single keyword (lowercase token as produced by the lexer stack).
    pub fn insert(&mut self, word: impl Into<String>, lexeme: Lexeme) {
        self.keywords.insert(word.into(), lexeme);
    }

    /// Merge `other` into `self`. If the same keyword exists in both lexicons, **`self` wins**
    /// (only keys not already present are inserted from `other`).
    pub fn merged_with(&self, other: &Lexicon) -> Lexicon {
        let mut out = self.keywords.clone();
        for (k, v) in &other.keywords {
            out.entry(k.clone()).or_insert(*v);
        }
        Lexicon { keywords: out }
    }

    pub(crate) fn get(&self, key: &str) -> Option<Lexeme> {
        self.keywords.get(key).copied()
    }
}

static DEFAULT_ENGLISH_LEXICON: OnceLock<Arc<Lexicon>> = OnceLock::new();

pub(crate) fn shared_english_lexicon() -> Arc<Lexicon> {
    Arc::clone(DEFAULT_ENGLISH_LEXICON.get_or_init(|| Arc::new(Lexicon::english())))
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
/// Enum for all valid tokens in the parse string
pub enum Lexeme {
    A,
    An,
    On,
    In,
    At,
    The,
    Dash,
    /// Calendar day offset from the anchor date (meaning depends on [`Lexicon`] keyword mapping).
    DayOffset(i32),
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

    // Suffixes
    ST,
    ND,
    RD,
    TH,
}

const LEXER_STACK_SIZE: usize = 20;

impl Lexeme {
    /// Lex using the built-in English [`Lexicon`] (cached, shared).
    pub fn lex_line(s: String) -> Result<Vec<Lexeme>, crate::Error> {
        Self::lex_line_with_lexicon(s, shared_english_lexicon().as_ref())
    }

    /// Lex a string into lexemes using the given keyword table.
    pub fn lex_line_with_lexicon(
        s: String,
        lexicon: &Lexicon,
    ) -> Result<Vec<Lexeme>, crate::Error> {
        let s = s.to_lowercase();

        let mut lexemes = Vec::new();
        let chars = s.chars();
        let mut stack = String::with_capacity(LEXER_STACK_SIZE);

        let push_lexeme = |stack: &mut String, ls: &mut Vec<Lexeme>| {
            if stack.is_empty() {
                Ok(())
            } else if let Some(l) = lexicon.get(stack.as_str()) {
                ls.push(l);
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

        for c in chars {
            if c.is_whitespace() {
                push_lexeme(&mut stack, &mut lexemes)?;
                continue;
            }

            if stack
                .chars()
                .last()
                .is_some_and(|sc| sc.is_ascii_digit() != c.is_ascii_digit())
            {
                push_lexeme(&mut stack, &mut lexemes)?;
            }

            if stack.len() == LEXER_STACK_SIZE {
                return Err(crate::Error::ParseError);
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
                '.' => {
                    push_lexeme(&mut stack, &mut lexemes)?;
                    lexemes.push(Lexeme::Dot);
                }
                _ => stack.push(c),
            }
        }

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

#[test]
fn test_lex_day_offset_keywords() {
    assert_eq!(
        Ok(vec![Lexeme::DayOffset(-1)]),
        Lexeme::lex_line("yesterday".into())
    );
    assert_eq!(
        Ok(vec![Lexeme::DayOffset(0)]),
        Lexeme::lex_line("today".into())
    );
    assert_eq!(
        Ok(vec![Lexeme::DayOffset(1)]),
        Lexeme::lex_line("tomorrow".into())
    );
    assert_eq!(
        Ok(vec![
            Lexeme::DayOffset(-1),
            Lexeme::DayOffset(0),
            Lexeme::DayOffset(1),
        ]),
        Lexeme::lex_line("yesterday today tomorrow".into())
    );
}

#[test]
fn test_lexicon_merge_first_wins() {
    let en = Lexicon::english();
    let mut rogue = Lexicon::empty();
    rogue.insert("today", Lexeme::DayOffset(99));
    let merged = en.merged_with(&rogue);
    assert_eq!(merged.get("today"), Some(Lexeme::DayOffset(0)));

    let merged_rev = rogue.merged_with(&en);
    assert_eq!(merged_rev.get("today"), Some(Lexeme::DayOffset(99)));
}

#[test]
fn test_dutch_day_offsets_lex() {
    let nl = Lexicon::dutch();
    assert_eq!(
        Ok(vec![Lexeme::DayOffset(2)]),
        Lexeme::lex_line_with_lexicon("overmorgen".into(), &nl)
    );
    assert_eq!(
        Ok(vec![Lexeme::DayOffset(-2)]),
        Lexeme::lex_line_with_lexicon("eergisteren".into(), &nl)
    );
}
