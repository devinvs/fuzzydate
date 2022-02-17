use chrono::{
    Local,
    NaiveTime as ChronoTime,
    NaiveDate as ChronoDate,
    Datelike,
    NaiveDateTime as ChronoDateTime,
    Duration as ChronoDuration,
    Weekday as ChronoWeekday, Timelike
};

use crate::lexer::Lexeme;

#[derive(Debug, Eq, PartialEq)]
/// Root of the Abstract Syntax Tree, represents a fully parsed DateTime
pub enum DateTime {
    /// Standard date and time
    DateTime(Date, Time),
    /// A duration after a datetime
    After(Duration, Box<DateTime>),
    /// A duration before a datetime
    Before(Duration, Box<DateTime>),
    /// A duration before the current datetime
    Ago(Duration),
    /// The current datetime
    Now
}

impl DateTime {
    /// Parse a datetime from a slice of lexemes
    pub fn parse(l: &[Lexeme]) -> Result<Option<(Self, usize)>, String> {
        let mut tokens = 0;

        if l.get(tokens) == Some(&Lexeme::A) || l.get(tokens) == Some(&Lexeme::The) || l.get(tokens) == Some(&Lexeme::An) {
            tokens += 1;

            if let Some((unit, t)) = Unit::parse(&l[tokens..]) {
                tokens += t;
                if Some(&Lexeme::After) == l.get(tokens) || Some(&Lexeme::From) == l.get(tokens) {
                    tokens += 1;
                    let datetime = DateTime::parse(&l[tokens..])?;

                    if datetime.is_none() {
                        return Err("Expected datetime".into());
                    }

                    let (datetime, t) = datetime.unwrap();
                    tokens += t;

                    Ok(Some((Self::After(Duration {unit, num: 1}, Box::new(datetime)), tokens)))
                } else if Some(&Lexeme::Before) == l.get(tokens) {
                    tokens += 1;
                    let datetime = DateTime::parse(&l[tokens..])?;

                    if datetime.is_none() {
                        return Err("Expected datetime".into());
                    }

                    let (datetime, t) = datetime.unwrap();
                    tokens += t;

                    Ok(Some((Self::After(Duration { unit, num: 1 }, Box::new(datetime)), tokens)))
                } else if Some(&Lexeme::Ago) == l.get(tokens) {
                    tokens += 1;
                    Ok(Some((Self::Ago(Duration{unit, num: 1}), tokens)))

                } else {
                    Err("Expected 'after' 'before' or 'ago'".into())
                }
            } else {
                Err("Expected Unit".into())
            }

        } else if l.get(tokens) == Some(&Lexeme::Now) {
            tokens += 1;
            return Ok(Some((Self::Now, tokens)))
        } else if let Ok(Some((dur, t))) = Duration::parse(&l[tokens..]) {
            tokens += t;

            if Some(&Lexeme::After) == l.get(tokens) || Some(&Lexeme::From) == l.get(tokens) {
                tokens += 1;

                let datetime = DateTime::parse(&l[tokens..])?;

                if datetime.is_none() {
                    return Err("Expected datetime".into());
                }

                let (datetime, t) = datetime.unwrap();
                tokens += t;

                Ok(Some((Self::After(dur, Box::new(datetime)), tokens)))
            } else if Some(&Lexeme::Before) == l.get(tokens) {
                tokens += 1;

                let datetime = DateTime::parse(&l[tokens..])?;

                if datetime.is_none() {
                    return Err("Expected datetime".into());
                }

                let (datetime, t) = datetime.unwrap();
                tokens += t;

                Ok(Some((Self::Before(dur, Box::new(datetime)), tokens)))
            } else if Some(&Lexeme::Ago) == l.get(tokens) {
                Ok(Some((Self::Ago(dur), tokens)))
            } else {
                Err("Expected 'after' or 'before'".into())
            }
        } else if let Some((date, t)) = Date::parse(&l[tokens..])? {
            tokens += t;
            if l.get(tokens) == Some(&Lexeme::Comma) {
                tokens += 1;
            }

            let (time, t) = Time::parse(&l[tokens..])?;
            tokens += t;

            Ok(Some((Self::DateTime(date, time), tokens)))
        } else {
            Ok(None)
        }
    }

    /// Convert a parsed DateTime to chrono's NaiveDateTime
    pub fn to_chrono(&self) -> ChronoDateTime {
        match self {
            DateTime::Now => Local::now().naive_local(),
            DateTime::DateTime(date, time) => {
                let date = date.to_chrono();
                let time = time.to_chrono();

                ChronoDateTime::new(date, time)
            }
            DateTime::After(dur, date) => {
                let mut date = date.to_chrono();

                if dur.convertable() {
                    date + dur.to_chrono()
                } else {
                    match dur.unit {
                        Unit::Month => {
                            if date.month() == 12 {
                                date = date.with_month(1).unwrap();
                                date.with_year(date.year() + dur.num as i32).unwrap()
                            } else {
                                date.with_month(date.month()+dur.num).unwrap()
                            }
                        }
                        Unit::Year => {
                            date.with_year(date.year()+dur.num as i32).unwrap()
                        }
                        _ => unreachable!()
                    }
                }
            }
            DateTime::Before(dur, date) => {
                let mut date = date.to_chrono();

                if dur.convertable() {
                    date - dur.to_chrono()
                } else {
                    match dur.unit {
                        Unit::Month => {
                            if date.month() == 12 {
                                date = date.with_month(1).unwrap();
                                date.with_year(date.year() - dur.num as i32).unwrap()
                            } else {
                                date.with_month(date.month()-dur.num).unwrap()
                            }
                        }
                        Unit::Year => {
                            date.with_year(date.year()-dur.num as i32).unwrap()
                        }
                        _ => unreachable!()
                    }
                }
            }
            DateTime::Ago(dur) => {
                let mut date = Local::now().naive_local();

                if dur.convertable() {
                    date - dur.to_chrono()
                } else {
                    match dur.unit {
                        Unit::Month => {
                            if date.month() == 12 {
                                date = date.with_month(1).unwrap();
                                date.with_year(date.year() - dur.num as i32).unwrap()
                            } else {
                                date.with_month(date.month()-dur.num).unwrap()
                            }
                        }
                        Unit::Year => {
                            date.with_year(date.year()-dur.num as i32).unwrap()
                        }
                        _ => unreachable!()
                    }
                }
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
/// A Parsed Date
pub enum Date {
    MonthNumDayYear(u32, u32, u32),
    MonthDayYear(Month, u32, u32),
    MonthNumDay(u32, u32),
    MonthDay(Month, u32),
    Relative(RelativeSpecifier, Weekday),
    Weekday(Weekday),
    Today,
    Tomorrow,
    Yesterday
}

impl Date {
    fn parse(l: &[Lexeme]) -> Result<Option<(Self, usize)>, String> {
        let mut tokens = 0;

        if let Some(&Lexeme::Today) = l.get(tokens) {
            tokens += 1;
            Ok(Some((Self::Today, tokens)))
        } else if let Some(&Lexeme::Tomorrow) = l.get(tokens) {
            tokens += 1;
            Ok(Some((Self::Tomorrow, tokens)))
        } else if let Some(&Lexeme::Yesterday) = l.get(tokens) {
            tokens += 1;
            Ok(Some((Self::Yesterday, tokens)))
        } else if let Some((month, t)) = Month::parse(&l[tokens..]) {
            tokens += t;
            if let Some((day, t)) = Num::parse(&l[tokens..])? {
                tokens += t;
                if l.get(tokens) == Some(&Lexeme::Comma) {
                    tokens += 1;
                }

                if let Some((year, t)) = Num::parse(&l[tokens..])? {
                    tokens += t;
                    Ok(Some((Self::MonthDayYear(month, day, year), tokens)))
                } else {
                    Ok(Some((Self::MonthDay(month, day), tokens)))
                }
            } else {
                Err("Expected day".into())
            }
        } else if let Some((relspec, t)) = RelativeSpecifier::parse(&l[tokens..]) {
            tokens += t;
            if let Some((weekday, t)) = Weekday::parse(&l[tokens..]) {
                tokens += t;
                Ok(Some((Self::Relative(relspec, weekday), tokens)))
            } else {
                Err("Expected weekday".into())
            }
        } else if let Some((weekday,t)) = Weekday::parse(&l[tokens..]) {
            tokens += t;
            Ok(Some((Self::Weekday(weekday), tokens)))
        } else if let Some((month, t)) = Num::parse(&l[tokens..])? {
            tokens += t;
            if l.get(tokens) != Some(&Lexeme::Slash)
            && l.get(tokens) != Some(&Lexeme::Dash)
            {
                return Err("expected / or -".into());
            }

            tokens += 1;

            if let Some((day, t)) = Num::parse(&l[tokens..])? {
                tokens += t;
                if l.get(tokens) == Some(&Lexeme::Slash)
                || l.get(tokens) == Some(&Lexeme::Dash)
                {
                    tokens += 1;
                    if let Some((year, t)) = Num::parse(&l[tokens..])? {
                        tokens += t;
                        Ok(Some((Self::MonthNumDayYear(month, day, year), tokens)))
                    } else {
                        Ok(Some((Self::MonthNumDay(month, day), tokens)))
                    }
                } else {
                    Ok(Some((Self::MonthNumDay(month, day), tokens)))
                }
            } else {
                Err("Expected day".into())
            }
        } else {
            Ok(None)
        }
    }

    fn to_chrono(&self) -> ChronoDate {
        match self {
            Date::Today => Local::now().naive_local().date(),
            Date::Yesterday => {
                let today = Local::now().naive_local().date();
                today - ChronoDuration::days(1)
            }
            Date::Tomorrow => {
                let today = Local::now().naive_local().date();
                today + ChronoDuration::days(1)
            }
            Date::MonthNumDay(month, day) => {
                let today = Local::now().naive_local().date();
                ChronoDate::from_ymd(today.year(), *month, *day)
            }
            Date::MonthNumDayYear(month, day, year) => {
                let curr = Local::now().naive_local().year() as u32;
                let year = if *year < 100 {
                    if curr+10 < 2000+*year {
                        1900 + *year
                    } else {
                        2000 + *year
                    }
                } else {
                    *year
                };

                ChronoDate::from_ymd(year as i32, *month, *day)
            }
            Date::MonthDay(month, day) => {
                let today = Local::now().naive_local().date();
                let month = *month as u32;
                ChronoDate::from_ymd(today.year(), month, *day)
            }
            Date::MonthDayYear(month, day, year) => {
                ChronoDate::from_ymd(*year as i32, *month as u32, *day)
            }
            Date::Relative(relspec, weekday) => {
                let weekday = weekday.to_chrono();
                let mut date = Local::now().naive_local().date();

                if relspec == &RelativeSpecifier::Next {
                    date += ChronoDuration::weeks(1);
                }

                if relspec == &RelativeSpecifier::Last {
                    date -= ChronoDuration::weeks(1);
                }

                while date.weekday() != weekday {
                    date += ChronoDuration::days(1);
                }

                date
            }
            Date::Weekday(weekday) => {
                let weekday = weekday.to_chrono();
                let mut date = Local::now().naive_local().date();

                while date.weekday() != weekday {
                    date += ChronoDuration::days(1);
                }

                date
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum RelativeSpecifier {
    This,
    Next,
    Last
}

impl RelativeSpecifier {
    fn parse(l: &[Lexeme]) -> Option<(Self, usize)> {
        let res = match l.get(0) {
            Some(Lexeme::This) => Some(Self::This),
            Some(Lexeme::Next) => Some(Self::Next),
            Some(Lexeme::Last) => Some(Self::Last),
            _ => None
        };

        res.map(|e| (e, 1))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday
}

impl Weekday {
    fn parse(l: &[Lexeme]) -> Option<(Self, usize)> {
        let res = match l.get(0) {
            Some(Lexeme::Sunday) => Some(Self::Sunday),
            Some(Lexeme::Monday) => Some(Self::Monday),
            Some(Lexeme::Tuesday) => Some(Self::Tuesday),
            Some(Lexeme::Wednesday) => Some(Self::Wednesday),
            Some(Lexeme::Thursday) => Some(Self::Thursday),
            Some(Lexeme::Friday) => Some(Self::Friday),
            Some(Lexeme::Saturday) => Some(Self::Saturday),
            _ => None
        };

        res.map(|e| (e, 1))
    }

    fn to_chrono(&self) -> ChronoWeekday {
        match *self {
            Weekday::Monday => ChronoWeekday::Mon,
            Weekday::Tuesday => ChronoWeekday::Tue,
            Weekday::Wednesday => ChronoWeekday::Wed,
            Weekday::Thursday => ChronoWeekday::Thu,
            Weekday::Friday => ChronoWeekday::Fri,
            Weekday::Saturday => ChronoWeekday::Sat,
            Weekday::Sunday => ChronoWeekday::Sun
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Month {
    January = 1,
    February = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12
}

impl Month {
    fn parse(l: &[Lexeme]) -> Option<(Self, usize)> {
        let res = match l.get(0) {
            Some(Lexeme::January) => Some(Self::January),
            Some(Lexeme::February) => Some(Self::February),
            Some(Lexeme::March) => Some(Self::March),
            Some(Lexeme::April) => Some(Self::April),
            Some(Lexeme::May) => Some(Self::May),
            Some(Lexeme::June) => Some(Self::June),
            Some(Lexeme::July) => Some(Self::July),
            Some(Lexeme::August) => Some(Self::August),
            Some(Lexeme::September) => Some(Self::September),
            Some(Lexeme::October) => Some(Self::October),
            Some(Lexeme::November) => Some(Self::November),
            Some(Lexeme::December) => Some(Self::December),
            _ => None
        };

        res.map(|e| (e, 1))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Time {
    HourMin(u32, u32),
    HourMinAM(u32, u32),
    HourMinPM(u32, u32),
    Empty
}

impl Time {
    fn parse(l: &[Lexeme]) -> Result<(Self, usize), String> {
        let mut tokens = 0;

        if let Some((hour, t)) = Num::parse(&l[tokens..])? {
            tokens += t;
            if l.get(tokens) != Some(&Lexeme::Colon) {
                return Err("Expected Colon".into());
            }

            tokens += 1;

            if let Some((min, t)) = Num::parse(&l[tokens..])? {
                tokens += t;
                if let Some(&Lexeme::AM) = l.get(tokens) {
                    tokens += 1;
                    Ok((Time::HourMinAM(hour, min), tokens))
                } else if let Some(&Lexeme::PM) = l.get(tokens) {
                    tokens += 1;
                    Ok((Time::HourMinPM(hour, min), tokens))
                } else {
                    Ok((Time::HourMin(hour, min), tokens))
                }
            } else {
                Err("Expected minute".into())
            }
        } else {
            Ok((Self::Empty, tokens))
        }
    }

    fn to_chrono(&self) -> ChronoTime {
        match *self {
            Time::Empty => Local::now().naive_local().time(),
            Time::HourMin(hour, min) => {
                ChronoTime::from_hms(hour, min, 0)
            }
            Time::HourMinAM(hour, min) => {
                ChronoTime::from_hms(hour, min, 0)
            }
            Time::HourMinPM(hour, min) => {
                ChronoTime::from_hms(hour+12, min, 0)
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Duration {
    num: u32,
    unit: Unit
}

impl Duration {
    fn parse(l: &[Lexeme]) -> Result<Option<(Self, usize)>, String> {
        let mut tokens = 0;

        if let Some((num, t)) = Num::parse(&l[tokens..])? {
            tokens += t;
            if let Some((u, t)) = Unit::parse(&l[tokens..]) {
                tokens += t;
                Ok(Some((Self {num, unit: u}, tokens)))
            } else {
                Err("Expected Unit while parsing Duration".into())
            }
        } else {
            Ok(None)
        }
    }

    fn convertable(&self) -> bool {
        self.unit != Unit::Month &&
        self.unit != Unit::Year
    }

    fn to_chrono(&self) -> ChronoDuration {
        match self.unit {
            Unit::Day => ChronoDuration::days(self.num as i64),
            Unit::Week => ChronoDuration::weeks(self.num as i64),
            Unit::Hour => ChronoDuration::hours(self.num as i64),
            Unit::Minute => ChronoDuration::minutes(self.num as i64),
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Unit {
    Day,
    Week,
    Hour,
    Minute,
    Month,
    Year
}

impl Unit {
    fn parse(l: &[Lexeme]) -> Option<(Self, usize)> {
        match l.get(0) {
            Some(Lexeme::Day) => Some((Unit::Day, 1)),
            Some(Lexeme::Week) => Some((Unit::Week, 1)),
            Some(Lexeme::Month) => Some((Unit::Month, 1)),
            Some(Lexeme::Year) => Some((Unit::Year, 1)),
            Some(Lexeme::Minute) => Some((Unit::Minute, 1)),
            Some(Lexeme::Hour) => Some((Unit::Hour, 1)),
            _ => None
        }
    }

}

struct Ones;

impl Ones {
    fn parse(l: &[Lexeme]) -> Option<(u32, usize)> {
        let mut res = match l.get(0) {
            Some(Lexeme::One) => Some(1),
            Some(Lexeme::Two) => Some(2),
            Some(Lexeme::Three) => Some(3),
            Some(Lexeme::Four) => Some(4),
            Some(Lexeme::Five) => Some(5),
            Some(Lexeme::Six) => Some(6),
            Some(Lexeme::Seven) => Some(7),
            Some(Lexeme::Eight) => Some(8),
            Some(Lexeme::Nine) => Some(9),
            _ => None
        };

        if res.is_none() {
            if let Some(Lexeme::Num(n)) = l.get(0) {
                if *n < 10 {
                    res = Some(*n);
                }
            }
        }

        res.map(|n| (n, 1))
    }
}

struct Teens;
impl Teens {
    fn parse(l: &[Lexeme]) -> Option<(u32, usize)> {
        let mut res = match l.get(0) {
            Some(Lexeme::Ten) => Some((10, 1)),
            Some(Lexeme::Eleven) => Some((11, 1)),
            Some(Lexeme::Twelve) => Some((12, 1)),
            Some(Lexeme::Thirteen) => Some((13, 1)),
            Some(Lexeme::Fourteen) => Some((14, 1)),
            Some(Lexeme::Fifteen) => Some((15, 1)),
            Some(Lexeme::Sixteen) => Some((16, 1)),
            Some(Lexeme::Seventeen) => Some((17, 1)),
            Some(Lexeme::Eighteen) => Some((18, 1)),
            Some(Lexeme::Nineteen) => Some((19, 1)),
            _ => None
        };

        if res.is_none() {
            if let Some(Lexeme::Num(n)) = l.get(0) {
                if *n >= 10 && *n <= 19 {
                    res = Some((*n, 1));
                }
            }
        }

        res
    }
}


struct Tens;
impl Tens {
    fn parse(l: &[Lexeme]) -> Option<(u32, usize)> {
        match l.get(0) {
            Some(Lexeme::Twenty) => Some((20, 1)),
            Some(Lexeme::Thirty) => Some((30, 1)),
            Some(Lexeme::Fourty) => Some((40, 1)),
            Some(Lexeme::Fifty) => Some((50, 1)),
            Some(Lexeme::Sixty) => Some((60, 1)),
            Some(Lexeme::Seventy) => Some((70, 1)),
            Some(Lexeme::Eighty) => Some((80, 1)),
            Some(Lexeme::Ninety) => Some((90, 1)),
            _ => None
        }
    }
}

struct NumDouble;
impl NumDouble {
    fn parse(l: &[Lexeme]) -> Option<(u32, usize)> {
        let mut tokens = 0;

        if let Some((tens, t)) = Tens::parse(&l[tokens..]) {
            tokens += t;
            if Some(&Lexeme::Dash) == l.get(tokens) {
                tokens += 1;
            }

            let (ones, t) = Ones::parse(&l[tokens..]).unwrap_or((0, 0));
            tokens += t;
            Some((tens + ones, tokens))
        } else if let Some((teens, t)) = Teens::parse(&l[tokens..]) {
            tokens += t;
            Some((teens, tokens))
        } else if let Some((ones, t)) = Ones::parse(&l[tokens..]) {
            tokens += t;
            Some((ones, tokens))
        } else {
            if let Some(Lexeme::Num(n)) = l.get(tokens) {
                tokens += 1;
                if *n < 100 && *n > 19 {
                    Some((*n, tokens))
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

struct NumTriple;
impl NumTriple {
    fn parse(l: &[Lexeme]) -> Result<Option<(u32, usize)>, String> {
        let mut tokens = 0;

        if let Some((ones, t)) = Ones::parse(&l[tokens..]) {
            tokens += t;

            if Some(&Lexeme::Hundred) != l.get(tokens) {
                // Try parsing instead as num_double
                tokens -= t;
                if let Some((double, t)) = NumDouble::parse(&l[tokens..]) {
                    return Ok(Some((double, t)));
                } else {
                    return Err("Expected 'hundred'".into());
                }
            }
            tokens += 1;

            let required = Some(&Lexeme::And) == l.get(tokens);
            if required { tokens += 1; }
            let double = NumDouble::parse(&l[tokens..]);

            if required && double.is_none() {
                return Err("Expected number after 'and'".into());
            }

            let (double, t) = double.unwrap_or((0, 0));
            tokens += t;

            Ok(Some((ones*100+double, tokens)))
        } else if Some(&Lexeme::Hundred) == l.get(tokens) {
            tokens += 1;

            let required = Some(&Lexeme::And) == l.get(tokens);
            if required { tokens += 1; }
            let double = NumDouble::parse(&l[tokens..]);

            if required && double.is_none() {
                return Err("Expected number after 'and'".into());
            }

            let (double, t) = double.unwrap_or((0, 0));
            tokens += t;

            Ok(Some((100+double, tokens)))
        } else if let Some((num_double, t)) = NumDouble::parse(&l[tokens..]) {
            tokens += t;
            Ok(Some((num_double, tokens)))
        } else if let Some(&Lexeme::Num(n)) = l.get(tokens) {
            tokens += 1;
            if n > 99 && n < 1000 {
                Ok(Some((n, tokens)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

struct NumTripleUnit;
impl NumTripleUnit {
    fn parse(l: &[Lexeme]) -> Option<(u32, usize)> {
        match l.get(0) {
            Some(Lexeme::Thousand) => Some((1000, 1)),
            Some(Lexeme::Million) => Some((1000000, 1)),
            Some(Lexeme::Billion) => Some((1000000000, 1)),
            _ => None
        }
    }
}

struct Num;
impl Num {
    fn parse(l: &[Lexeme]) -> Result<Option<(u32, usize)>, String> {
        let mut tokens = 0;

        if let Some((triple, t)) = NumTriple::parse(&l[tokens..])? {
            tokens += t;
            let unit = NumTripleUnit::parse(&l[tokens..]);

            if unit.is_none() {
                return Ok(Some((triple, tokens)))
            }

            let (unit, t) = unit.unwrap();
            tokens += t;

            let required = Some(&Lexeme::And) == l.get(tokens);
            if required { tokens += 1; }
            let num = Num::parse(&l[tokens..])?;

            if required && num.is_none() {
                return Err("Expected num".into());
            }

            let (num, t) = num.unwrap_or((0, 0));
            tokens += t;

            Ok(Some((triple*unit+num, tokens)))
        } else if let Some((unit, t)) = NumTripleUnit::parse(&l[tokens..]) {
            tokens += t;

            let required = Some(&Lexeme::And) == l.get(tokens);
            if required { tokens += 1; }
            let num = Num::parse(&l[tokens..])?;

            if required && num.is_none() {
                return Err("Expected num".into());
            }

            let (num, t) = num.unwrap_or((0, 0));
            tokens += t;

            Ok(Some((unit + num, tokens)))
        } else if let Some(&Lexeme::Num(n)) = l.get(tokens) {
            tokens += 1;
            if n >= 1000 {
                Ok(Some((n, tokens)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}


#[test]
fn test_ones() {
    let lexemes = vec![Lexeme::Five];
    let (ones, t) = Ones::parse(lexemes.as_slice()).unwrap();

    assert_eq!(ones, 5);
    assert_eq!(t, 1);
}

#[test]
fn test_ones_literal() {
    let lexemes = vec![Lexeme::Num(5)];
    let (ones, t) = Ones::parse(lexemes.as_slice()).unwrap();

    assert_eq!(ones, 5);
    assert_eq!(t, 1);
}

#[test]
fn test_simple_num() {
    let lexemes = vec![Lexeme::Num(5)];
    let (num, t) = Num::parse(lexemes.as_slice()).unwrap().unwrap();

    assert_eq!(num, 5);
    assert_eq!(t, 1);
}

#[test]
fn test_complex_triple_num() {
    let lexemes = vec![
        Lexeme::Num(2),
        Lexeme::Hundred,
        Lexeme::And,
        Lexeme::Thirty,
        Lexeme::Dash,
        Lexeme::Five
    ];
    let (num, t) = NumTriple::parse(lexemes.as_slice()).unwrap().unwrap();

    assert_eq!(num, 235);
    assert_eq!(t, 6);
}

#[test]
fn test_complex_num() {
    let lexemes = vec![
        Lexeme::Two,
        Lexeme::Hundred,
        Lexeme::Five,
        Lexeme::Million,
        Lexeme::Thirty,
        Lexeme::Thousand,
        Lexeme::And,
        Lexeme::Ten
    ];
    let (num, t) = Num::parse(lexemes.as_slice()).unwrap().unwrap();

    assert_eq!(num, 205_030_010);
    assert_eq!(t, 8)
}

#[test]
fn test_simple_date_time() {
    let lexemes = vec![
        Lexeme::February,
        Lexeme::Num(16),
        Lexeme::Num(2022),
        Lexeme::Num(5),
        Lexeme::Colon,
        Lexeme::Num(27),
        Lexeme::PM
    ];
    let (date,t) = DateTime::parse(lexemes.as_slice()).unwrap().unwrap();
    let date = date.to_chrono();

    assert_eq!(t, 7);
    assert_eq!(date.year(), 2022);
    assert_eq!(date.month(), 2);
    assert_eq!(date.day(), 16);
    assert_eq!(date.hour(), 17);
    assert_eq!(date.minute(), 27);
}

#[test]
fn test_complex_relative_datetime() {
    let lexemes = vec![
        Lexeme::A,
        Lexeme::Week,
        Lexeme::After,
        Lexeme::Two,
        Lexeme::Day,
        Lexeme::Before,
        Lexeme::The,
        Lexeme::Day,
        Lexeme::After,
        Lexeme::Tomorrow
    ];
    let today = Local::now().naive_local().date();
    let (date, t) = DateTime::parse(lexemes.as_slice()).unwrap().unwrap();
    let date = date.to_chrono();

    assert_eq!(t, 10);
    assert_eq!(date.year(), today.year());
    assert_eq!(date.month(), today.month());
    assert_eq!(date.day(), today.day() + 7 - 2 + 1 + 1);
}
