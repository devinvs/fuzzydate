use chrono::{
    Datelike, Duration as ChronoDuration, Local, NaiveDate as ChronoDate,
    NaiveDateTime as ChronoDateTime, NaiveTime as ChronoTime, Weekday as ChronoWeekday,
};

use crate::lexer::Lexeme;

#[derive(Debug, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
/// Root of the Abstract Syntax Tree, represents a fully parsed DateTime
pub enum DateTime {
    /// Standard date and time
    DateTime(Date, Time),
    /// Backwards
    TimeDate(Time, Date),
    /// A duration after a datetime
    After(Duration, Box<DateTime>),
    /// A duration before a datetime
    Before(Duration, Box<DateTime>),
    /// A duration before the current datetime
    Ago(Duration),
    /// The current datetime
    Now,
}

impl DateTime {
    /// Parse a datetime from a slice of lexemes
    pub fn parse(l: &[Lexeme]) -> Option<(Self, usize)> {
        let mut tokens = 0;
        if l.get(tokens) == Some(&Lexeme::Now) {
            tokens += 1;
            return Some((Self::Now, tokens));
        }

        tokens = 0;
        if let Some((dur, t)) = Duration::parse(&l[tokens..]) {
            tokens += t;

            if Some(&Lexeme::After) == l.get(tokens) || Some(&Lexeme::From) == l.get(tokens) {
                tokens += 1;

                if let Some((datetime, t)) = DateTime::parse(&l[tokens..]) {
                    tokens += t;
                    return Some((Self::After(dur, Box::new(datetime)), tokens));
                }
            } else if Some(&Lexeme::Before) == l.get(tokens) {
                tokens += 1;

                if let Some((datetime, t)) = DateTime::parse(&l[tokens..]) {
                    tokens += t;
                    return Some((Self::Before(dur, Box::new(datetime)), tokens));
                }
            } else if Some(&Lexeme::Ago) == l.get(tokens) {
                tokens += 1;
                return Some((Self::Ago(dur), tokens));
            }
        }

        tokens = 0;
        if let Some((date, t)) = Date::parse(&l[tokens..]) {
            tokens += t;
            if l.get(tokens) == Some(&Lexeme::Comma) {
                tokens += 1;
            }

            if let Some((time, t)) = Time::parse(&l[tokens..]) {
                tokens += t;
                return Some((Self::DateTime(date, time), tokens));
            }
        }

        tokens = 0;
        if let Some((time, t)) = Time::parse(&l[tokens..]) {
            tokens += t;
            if l.get(tokens) == Some(&Lexeme::Comma) {
                tokens += 1;
            }

            if let Some((date, t)) = Date::parse(&l[tokens..]) {
                tokens += t;
                return Some((Self::TimeDate(time, date), tokens));
            }
        }

        None
    }

    /// Convert a parsed DateTime to chrono's NaiveDateTime
    pub fn to_chrono(
        &self,
        default: ChronoTime,
        relative_to: Option<ChronoDateTime>,
    ) -> Result<ChronoDateTime, crate::Error> {
        let now = relative_to.unwrap_or(Local::now().naive_local());
        Ok(match self {
            DateTime::Now => now,
            DateTime::DateTime(date, time) => {
                let date = date.to_chrono(Some(now.date()))?;
                let time = time.to_chrono(default)?;

                ChronoDateTime::new(date, time)
            }
            DateTime::TimeDate(time, date) => {
                let date = date.to_chrono(Some(now.date()))?;
                let time = time.to_chrono(default)?;

                ChronoDateTime::new(date, time)
            }
            DateTime::After(dur, date) => {
                let date = date.to_chrono(default, relative_to)?;
                dur.after(date)
            }
            DateTime::Before(dur, date) => {
                let date = date.to_chrono(default, relative_to)?;
                dur.before(date)
            }
            DateTime::Ago(dur) => dur.before(now),
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
/// A Parsed Date
pub enum Date {
    MonthNumDayYear(u32, u32, u32),
    MonthDayYear(Month, u32, u32),
    MonthNumDay(u32, u32),
    MonthDay(Month, u32),
    UnitRelative(RelativeSpecifier, Unit),
    Relative(RelativeSpecifier, Weekday),
    Weekday(Weekday),
    Today,
    Tomorrow,
    Yesterday,
}

impl Date {
    fn parse(l: &[Lexeme]) -> Option<(Self, usize)> {
        let mut tokens = 0;

        if let Some(&Lexeme::Today) = l.get(tokens) {
            tokens += 1;
            return Some((Self::Today, tokens));
        }

        tokens = 0;
        if let Some(&Lexeme::Tomorrow) = l.get(tokens) {
            tokens += 1;
            return Some((Self::Tomorrow, tokens));
        }

        tokens = 0;
        if let Some(&Lexeme::Yesterday) = l.get(tokens) {
            tokens += 1;
            return Some((Self::Yesterday, tokens));
        }

        tokens = 0;
        if let Some((month, t)) = Month::parse(&l[tokens..]) {
            tokens += t;

            let (day, t) = Num::parse(&l[tokens..])?;
            tokens += t;

            if let Some((year, t)) = Num::parse(&l[tokens..]) {
                tokens += t;
                return Some((Self::MonthDayYear(month, day, year), tokens));
            } else {
                return Some((Self::MonthDay(month, day), tokens));
            }
        }

        tokens = 0;
        if let Some((relspec, t)) = RelativeSpecifier::parse(&l[tokens..]) {
            tokens += t;

            if let Some((weekday, t)) = Weekday::parse(&l[tokens..]) {
                tokens += t;
                return Some((Self::Relative(relspec, weekday), tokens));
            }

            if let Some((unit, t)) = Unit::parse(&l[tokens..]) {
                tokens += t;
                return Some((Self::UnitRelative(relspec, unit), tokens));
            }
        } else if let Some((weekday, t)) = Weekday::parse(&l[tokens..]) {
            tokens += t;
            return Some((Self::Weekday(weekday), tokens));
        } else if let Some((num1, t)) = Num::parse(&l[tokens..]) {
            tokens += t;
            if let Some(delim) = l.get(tokens) {
                if delim == &Lexeme::Slash || delim == &Lexeme::Dash || delim == &Lexeme::Dot {
                    // Consume slash or dash
                    tokens += 1;

                    if let Some((num2, t)) = Num::parse(&l[tokens..]) {
                        tokens += t;
                        if l.get(tokens)? == delim {
                            // Consume slash or dash
                            tokens += 1;

                            let (num3, t) = Num::parse(&l[tokens..])?;
                            tokens += t;

                            // If delim is dot use DMY, otherwise MDY
                            if delim == &Lexeme::Dot {
                                return Some((Self::MonthNumDayYear(num2, num1, num3), tokens));
                            } else {
                                return Some((Self::MonthNumDayYear(num1, num2, num3), tokens));
                            }
                        } else {
                            // If delim is dot use DMY, otherwise MDY
                            if delim == &Lexeme::Dot {
                                return Some((Self::MonthNumDay(num2, num1), tokens));
                            } else {
                                return Some((Self::MonthNumDay(num1, num2), tokens));
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn to_chrono(&self, relative_to: Option<ChronoDate>) -> Result<ChronoDate, crate::Error> {
        let mut today = relative_to.unwrap_or(Local::now().naive_local().date());
        Ok(match self {
            Date::Today => today,
            Date::Yesterday => today - ChronoDuration::days(1),
            Date::Tomorrow => today + ChronoDuration::days(1),
            Date::MonthNumDay(month, day) => ChronoDate::from_ymd_opt(today.year(), *month, *day)
                .ok_or(crate::Error::InvalidDate(format!(
                "Invalid month-day: {month}-{day}"
            )))?,
            Date::MonthNumDayYear(month, day, year) => {
                let curr = today.year() as u32;
                let year = if *year < 100 {
                    if curr + 10 < 2000 + *year {
                        1900 + *year
                    } else {
                        2000 + *year
                    }
                } else {
                    *year
                };

                ChronoDate::from_ymd_opt(year as i32, *month, *day).ok_or(
                    crate::Error::InvalidDate(format!(
                        "Invalid year-month-day: {year}-{month}-{day}"
                    )),
                )?
            }
            Date::MonthDay(month, day) => {
                let month = *month as u32;
                ChronoDate::from_ymd_opt(today.year(), month, *day).ok_or(
                    crate::Error::InvalidDate(format!("Invalid month-day: {month}-{day}")),
                )?
            }
            Date::MonthDayYear(month, day, year) => {
                ChronoDate::from_ymd_opt(*year as i32, *month as u32, *day).ok_or(
                    crate::Error::InvalidDate(format!(
                        "Invalid year-month-day: {}-{}-{}",
                        *year, *month as u32, *day
                    )),
                )?
            }
            Date::Relative(relspec, weekday) => {
                let weekday = weekday.to_chrono();

                if relspec == &RelativeSpecifier::Next {
                    today += ChronoDuration::weeks(1);
                }

                if relspec == &RelativeSpecifier::Last {
                    today -= ChronoDuration::weeks(1);
                }

                while today.weekday() != weekday {
                    today += ChronoDuration::days(1);
                }

                today
            }
            Date::UnitRelative(relspec, unit) => {
                let mut date = today;
                if relspec == &RelativeSpecifier::Next {
                    date = Duration::Specific(1, unit.to_owned())
                        .after(today.into())
                        .date();
                }

                if relspec == &RelativeSpecifier::Last {
                    date = Duration::Specific(1, unit.to_owned())
                        .before(today.into())
                        .date();
                }

                date
            }
            Date::Weekday(weekday) => {
                let weekday = weekday.to_chrono();
                let mut date = today;

                while date.weekday() != weekday {
                    date += ChronoDuration::days(1);
                }

                date
            }
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum RelativeSpecifier {
    This,
    Next,
    Last,
}

impl RelativeSpecifier {
    fn parse(l: &[Lexeme]) -> Option<(Self, usize)> {
        let res = match l.get(0) {
            Some(Lexeme::This) => Some(Self::This),
            Some(Lexeme::Next) => Some(Self::Next),
            Some(Lexeme::Last) => Some(Self::Last),
            _ => None,
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
    Sunday,
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
            _ => None,
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
            Weekday::Sunday => ChronoWeekday::Sun,
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
    December = 12,
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
            _ => None,
        };

        res.map(|e| (e, 1))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Time {
    HourMin(u32, u32),
    HourMinAM(u32, u32),
    HourMinPM(u32, u32),
    Empty,
}

impl Time {
    fn parse(l: &[Lexeme]) -> Option<(Self, usize)> {
        let mut tokens = 0;

        if let Some((hour, t)) = Num::parse(&l[tokens..]) {
            tokens += t;
            if l.get(tokens) == Some(&Lexeme::Colon) {
                tokens += 1;

                if let Some((min, t)) = Num::parse(&l[tokens..]) {
                    tokens += t;
                    if let Some(&Lexeme::AM) = l.get(tokens) {
                        tokens += 1;
                        return Some((Time::HourMinAM(hour, min), tokens));
                    } else if let Some(&Lexeme::PM) = l.get(tokens) {
                        tokens += 1;
                        return Some((Time::HourMinPM(hour, min), tokens));
                    } else {
                        return Some((Time::HourMin(hour, min), tokens));
                    }
                }
            }
        }

        tokens = 0;
        Some((Self::Empty, tokens))
    }

    fn to_chrono(&self, default: ChronoTime) -> Result<ChronoTime, crate::Error> {
        match *self {
            Time::Empty => Ok(default),
            Time::HourMin(hour, min) => ChronoTime::from_hms_opt(hour, min, 0).ok_or(
                crate::Error::InvalidDate(format!("Invalid time: {hour}:{min}")),
            ),
            Time::HourMinAM(hour, min) => ChronoTime::from_hms_opt(hour, min, 0).ok_or(
                crate::Error::InvalidDate(format!("Invalid time: {hour}:{min} am")),
            ),
            Time::HourMinPM(hour, min) => ChronoTime::from_hms_opt(hour + 12, min, 0).ok_or(
                crate::Error::InvalidDate(format!("Invalid time: {hour}:{min} pm")),
            ),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Article {
    A,
    An,
    The,
}

impl Article {
    fn parse(l: &[Lexeme]) -> Option<(Self, usize)> {
        match l.get(0) {
            Some(Lexeme::A) => Some((Self::A, 1)),
            Some(Lexeme::An) => Some((Self::An, 1)),
            Some(Lexeme::The) => Some((Self::The, 1)),
            _ => None,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Duration {
    Article(Unit),
    Specific(u32, Unit),
    Concat(Box<Duration>, Box<Duration>),
}

impl Duration {
    fn parse(l: &[Lexeme]) -> Option<(Self, usize)> {
        let mut tokens = 0;
        if let Some((d, t)) = Duration::parse_concrete(l) {
            tokens += t;

            if let Some(Lexeme::And) = l.get(tokens) {
                tokens += 1;

                if let Some((dur2, t)) = Duration::parse(&l[tokens..]) {
                    tokens += t;

                    return Some((Duration::Concat(Box::new(d), Box::new(dur2)), tokens));
                }
            }

            return Some((d, t));
        }

        None
    }

    fn parse_concrete(l: &[Lexeme]) -> Option<(Self, usize)> {
        let mut tokens = 0;

        if let Some((num, t)) = Num::parse(&l[tokens..]) {
            tokens += t;
            if let Some((u, t)) = Unit::parse(&l[tokens..]) {
                tokens += t;
                return Some((Self::Specific(num, u), tokens));
            }
        }

        tokens = 0;
        if let Some((_, t)) = Article::parse(l) {
            tokens += t;
            if let Some((u, t)) = Unit::parse(&l[tokens..]) {
                tokens += t;
                return Some((Self::Article(u), tokens));
            }
        }

        None
    }

    fn unit(&self) -> &Unit {
        match self {
            Duration::Article(u) => u,
            Duration::Specific(_, u) => u,
            _ => unimplemented!(),
        }
    }

    fn num(&self) -> u32 {
        match *self {
            Duration::Article(_) => 1,
            Duration::Specific(num, _) => num,
            _ => unimplemented!(),
        }
    }

    fn convertable(&self) -> bool {
        if let Duration::Concat(dur1, dur2) = self {
            return dur1.convertable() && dur2.convertable();
        }

        let unit = self.unit();
        unit != &Unit::Month && unit != &Unit::Year
    }

    fn to_chrono(&self) -> ChronoDuration {
        if let Duration::Concat(dur1, dur2) = self {
            return dur1.to_chrono() + dur2.to_chrono();
        }

        let unit = self.unit();
        let num = self.num();

        match unit {
            Unit::Day => ChronoDuration::days(num as i64),
            Unit::Week => ChronoDuration::weeks(num as i64),
            Unit::Hour => ChronoDuration::hours(num as i64),
            Unit::Minute => ChronoDuration::minutes(num as i64),
            _ => unreachable!(),
        }
    }

    fn after(&self, date: ChronoDateTime) -> ChronoDateTime {
        if let Duration::Concat(dur1, dur2) = self {
            return dur2.after(dur1.after(date));
        }

        if self.convertable() {
            date + self.to_chrono()
        } else {
            match self.unit() {
                Unit::Month => date
                    .checked_add_months(chrono::Months::new(self.num()))
                    .expect("Date out of representable date range."),
                Unit::Year => date.with_year(date.year() + self.num() as i32).unwrap(),
                _ => unreachable!(),
            }
        }
    }

    fn before(&self, date: ChronoDateTime) -> ChronoDateTime {
        if let Duration::Concat(dur1, dur2) = self {
            return dur2.before(dur1.before(date));
        }

        if self.convertable() {
            date - self.to_chrono()
        } else {
            match self.unit() {
                Unit::Month => date
                    .checked_sub_months(chrono::Months::new(self.num()))
                    .expect("Date out of representable date range."),
                Unit::Year => date.with_year(date.year() - self.num() as i32).unwrap(),
                _ => unreachable!(),
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Unit {
    Day,
    Week,
    Hour,
    Minute,
    Month,
    Year,
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
            _ => None,
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
            _ => None,
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
            _ => None,
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
            _ => None,
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
            return Some((tens + ones, tokens));
        }

        tokens = 0;
        if let Some((teens, t)) = Teens::parse(&l[tokens..]) {
            tokens += t;
            return Some((teens, tokens));
        }

        tokens = 0;
        if let Some((ones, t)) = Ones::parse(&l[tokens..]) {
            tokens += t;
            return Some((ones, tokens));
        }

        tokens = 0;
        if let Some(Lexeme::Num(n)) = l.get(tokens) {
            tokens += 1;
            if *n < 100 && *n > 19 {
                return Some((*n, tokens));
            }
        }

        None
    }
}

struct NumTriple;
impl NumTriple {
    fn parse(l: &[Lexeme]) -> Option<(u32, usize)> {
        let mut tokens = 0;

        if let Some((ones, t)) = Ones::parse(&l[tokens..]) {
            tokens += t;

            if Some(&Lexeme::Hundred) == l.get(tokens) {
                // Consume 'Hundred'
                tokens += 1;

                let required = Some(&Lexeme::And) == l.get(tokens);
                if required {
                    tokens += 1;
                }
                let double = NumDouble::parse(&l[tokens..]);

                if !required || double.is_some() {
                    let (double, t) = double.unwrap_or((0, 0));
                    tokens += t;

                    return Some((ones * 100 + double, tokens));
                }
            }
        }

        tokens = 0;
        if Some(&Lexeme::Hundred) == l.get(tokens) {
            tokens += 1;

            let required = Some(&Lexeme::And) == l.get(tokens);
            if required {
                tokens += 1;
            }
            let double = NumDouble::parse(&l[tokens..]);

            if !required || double.is_some() {
                let (double, t) = double.unwrap_or((0, 0));
                tokens += t;

                return Some((100 + double, tokens));
            }
        }

        tokens = 0;
        if let Some((num_double, t)) = NumDouble::parse(&l[tokens..]) {
            tokens += t;
            return Some((num_double, tokens));
        }

        tokens = 0;
        if let Some(&Lexeme::Num(n)) = l.get(tokens) {
            tokens += 1;
            if n > 99 && n < 1000 {
                return Some((n, tokens));
            }
        }

        None
    }
}

struct NumTripleUnit;
impl NumTripleUnit {
    fn parse(l: &[Lexeme]) -> Option<(u32, usize)> {
        match l.get(0) {
            Some(Lexeme::Thousand) => Some((1000, 1)),
            Some(Lexeme::Million) => Some((1000000, 1)),
            Some(Lexeme::Billion) => Some((1000000000, 1)),
            _ => None,
        }
    }
}

struct Num;
impl Num {
    fn parse(l: &[Lexeme]) -> Option<(u32, usize)> {
        let mut tokens = 0;

        // <num_triple>
        if let Some((triple, t)) = NumTriple::parse(&l[tokens..]) {
            tokens += t;

            // <num_triple_unit>
            if let Some((unit, t)) = NumTripleUnit::parse(&l[tokens..]) {
                tokens += t;

                let required = Some(&Lexeme::And) == l.get(tokens);
                if required {
                    tokens += 1;
                } // Consume and
                let num = Num::parse(&l[tokens..]);

                if !required || num.is_some() {
                    let (num, t) = num.unwrap_or((0, 0));
                    tokens += t;

                    return Some((triple * unit + num, tokens));
                }
            }
        }

        tokens = 0;
        // <num_triple_unit>
        if let Some((unit, t)) = NumTripleUnit::parse(&l[tokens..]) {
            tokens += t;

            let required = Some(&Lexeme::And) == l.get(tokens);
            if required {
                tokens += 1;
            } // Consume and
            let num = Num::parse(&l[tokens..]);

            if num.is_some() || !required {
                let (num, t) = num.unwrap_or((0, 0));
                tokens += t;

                return Some((unit + num, tokens));
            }
        }

        // <num_triple>
        tokens = 0;
        if let Some((num, t)) = NumTriple::parse(&l[tokens..]) {
            tokens += t;
            return Some((num, tokens));
        }

        tokens = 0;
        // NUM
        if let Some(&Lexeme::Num(n)) = l.get(tokens) {
            tokens += 1;
            if n >= 1000 {
                return Some((n, tokens));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDateTime as ChronoDateTime, TimeZone};
    use test_case::test_case;

    use crate::ast::*;
    use crate::lexer::Lexeme;

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
        let (num, t) = Num::parse(lexemes.as_slice()).unwrap();

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
            Lexeme::Five,
        ];
        let (num, t) = NumTriple::parse(lexemes.as_slice()).unwrap();

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
            Lexeme::Ten,
        ];
        let (num, t) = Num::parse(lexemes.as_slice()).unwrap();

        assert_eq!(t, 8);
        assert_eq!(num, 205_030_010);
    }

    #[test]
    fn test_simple_date_time() {
        use chrono::Timelike;

        let lexemes = vec![
            Lexeme::February,
            Lexeme::Num(16),
            Lexeme::Num(2022),
            Lexeme::Num(5),
            Lexeme::Colon,
            Lexeme::Num(27),
            Lexeme::PM,
        ];
        let (date, t) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date
            .to_chrono(Local::now().naive_local().time(), None)
            .unwrap();

        assert_eq!(t, 7);
        assert_eq!(date.year(), 2022);
        assert_eq!(date.month(), 2);
        assert_eq!(date.day(), 16);
        assert_eq!(date.hour(), 17);
        assert_eq!(date.minute(), 27);
    }

    #[test_case(None; "default reference time")]
    #[test_case(Some(Local.with_ymd_and_hms(2021, 4, 30, 7, 15, 17).single().expect("literal date for test case").naive_local()); "past reference time")]
    fn test_complex_relative_datetime(now: Option<ChronoDateTime>) {
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
            Lexeme::Tomorrow,
            Lexeme::Comma,
            Lexeme::Num(5),
            Lexeme::Colon,
            Lexeme::Num(20),
        ];

        use chrono::naive::Days;
        let today = now.map_or(Local::now().naive_local().date(), |now| now.date());
        let real_date = today + Days::new(7 - 2 + 1 + 1);

        let (date, t) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date
            .to_chrono(Local::now().naive_local().time(), now)
            .unwrap();

        assert_eq!(t, 14);
        assert_eq!(date.year(), real_date.year());
        assert_eq!(date.month(), real_date.month());
        assert_eq!(date.day(), real_date.day());
    }

    #[test_case(None; "default reference time")]
    #[test_case(Some(Local.with_ymd_and_hms(2021, 4, 30, 7, 15, 17).single().expect("literal date for test case").naive_local()); "past reference time")]
    fn test_datetime_now(now: Option<ChronoDateTime>) {
        use chrono::Timelike;

        let lexemes = vec![Lexeme::Now];
        let (date, t) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date
            .to_chrono(Local::now().naive_local().time(), now)
            .unwrap();

        let now = now.unwrap_or(Local::now().naive_local());
        assert_eq!(t, 1);
        assert_eq!(date.year(), now.year());
        assert_eq!(date.month(), now.month());
        assert_eq!(date.day(), now.day());
        assert_eq!(date.hour(), now.hour());
        assert_eq!(date.minute(), now.minute());
    }

    #[test]
    fn test_malformed_article_after() {
        let lexemes = vec![Lexeme::A, Lexeme::Day, Lexeme::After, Lexeme::Colon];
        assert!(DateTime::parse(lexemes.as_slice()).is_none());
    }

    #[test]
    fn test_malformed_after() {
        let lexemes = vec![Lexeme::Num(5), Lexeme::Day, Lexeme::After, Lexeme::Colon];
        assert!(DateTime::parse(lexemes.as_slice()).is_none());
    }

    #[test_case(None; "default reference time")]
    #[test_case(Some(Local.with_ymd_and_hms(2021, 4, 30, 7, 15, 17).single().expect("literal date for test case").naive_local()); "past reference time")]
    fn test_datetime_ago(now: Option<ChronoDateTime>) {
        let lexemes = vec![Lexeme::A, Lexeme::Day, Lexeme::Ago];
        let (date, t) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date
            .to_chrono(Local::now().naive_local().time(), now)
            .unwrap();

        let today = now.map_or(Local::now().naive_local().date(), |now| now.date());
        assert_eq!(t, 3);
        assert_eq!(date.year(), today.year());
        assert_eq!(date.month(), today.month());
        assert_eq!(date.day(), today.day() - 1);
    }

    #[test]
    fn test_teens() {
        assert_eq!((10, 1), Teens::parse(&[Lexeme::Ten]).unwrap());
        assert_eq!((11, 1), Teens::parse(&[Lexeme::Eleven]).unwrap());
        assert_eq!((12, 1), Teens::parse(&[Lexeme::Twelve]).unwrap());
        assert_eq!((13, 1), Teens::parse(&[Lexeme::Thirteen]).unwrap());
        assert_eq!((14, 1), Teens::parse(&[Lexeme::Fourteen]).unwrap());
        assert_eq!((15, 1), Teens::parse(&[Lexeme::Fifteen]).unwrap());
        assert_eq!((16, 1), Teens::parse(&[Lexeme::Sixteen]).unwrap());
        assert_eq!((17, 1), Teens::parse(&[Lexeme::Seventeen]).unwrap());
        assert_eq!((18, 1), Teens::parse(&[Lexeme::Eighteen]).unwrap());
        assert_eq!((19, 1), Teens::parse(&[Lexeme::Nineteen]).unwrap());
    }

    #[test_case(None; "default reference time")]
    #[test_case(Some(Local.with_ymd_and_hms(2021, 4, 30, 7, 15, 17).single().expect("literal date for test case").naive_local()); "past reference time")]
    fn test_article_before(now: Option<ChronoDateTime>) {
        let (date, t) =
            DateTime::parse(&[Lexeme::A, Lexeme::Day, Lexeme::Before, Lexeme::Today]).unwrap();
        let date = date
            .to_chrono(Local::now().naive_local().time(), now)
            .unwrap();

        let today = now.map_or(Local::now().naive_local().date(), |now| now.date());
        assert_eq!(t, 4);
        assert_eq!(date.year(), today.year());
        assert_eq!(date.month(), today.month());
        assert_eq!(date.day(), today.day() - 1);
    }

    #[test_case(None; "default reference time")]
    #[test_case(Some(Local.with_ymd_and_hms(2021, 4, 30, 7, 15, 17).single().expect("literal date for test case").naive_local()); "past reference time")]
    fn test_after_december(now: Option<ChronoDateTime>) {
        let l = vec![
            Lexeme::A,
            Lexeme::Month,
            Lexeme::After,
            Lexeme::December,
            Lexeme::Num(5),
        ];

        let today = now.map_or(Local::now().naive_local().date(), |now| now.date());
        let (date, t) = DateTime::parse(l.as_slice()).unwrap();
        let date = date
            .to_chrono(Local::now().naive_local().time(), now)
            .unwrap();

        assert_eq!(t, 5);
        assert_eq!(date.year(), today.year() + 1);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 5);
    }

    #[test_case(None; "default reference time")]
    #[test_case(Some(Local.with_ymd_and_hms(2021, 4, 30, 7, 15, 17).single().expect("literal date for test case").naive_local()); "past reference time")]
    fn test_month_before_january(now: Option<ChronoDateTime>) {
        let l = vec![
            Lexeme::A,
            Lexeme::Month,
            Lexeme::Before,
            Lexeme::January,
            Lexeme::Num(5),
        ];

        let today = now.map_or(Local::now().naive_local().date(), |now| now.date());
        let (date, t) = DateTime::parse(l.as_slice()).unwrap();
        let date = date
            .to_chrono(Local::now().naive_local().time(), now)
            .unwrap();

        assert_eq!(t, 5);
        assert_eq!(date.year(), today.year() - 1);
        assert_eq!(date.month(), 12);
        assert_eq!(date.day(), 5);
    }

    #[test_case(None; "default reference time")]
    #[test_case(Some(Local.with_ymd_and_hms(2021, 4, 30, 7, 15, 17).single().expect("literal date for test case").naive_local()); "past reference time")]
    fn test_week_after(now: Option<ChronoDateTime>) {
        let l = vec![
            Lexeme::A,
            Lexeme::Week,
            Lexeme::After,
            Lexeme::October,
            Lexeme::Num(5),
        ];

        let today = now.map_or(Local::now().naive_local().date(), |now| now.date());
        let (date, t) = DateTime::parse(l.as_slice()).unwrap();
        let date = date
            .to_chrono(Local::now().naive_local().time(), now)
            .unwrap();

        assert_eq!(t, 5);
        assert_eq!(date.year(), today.year());
        assert_eq!(date.month(), 10);
        assert_eq!(date.day(), 12);
    }

    #[test_case(None; "default reference time")]
    #[test_case(Some(Local.with_ymd_and_hms(2021, 4, 30, 7, 15, 17).single().expect("literal date for test case").naive_local()); "past reference time")]
    fn test_month_after(now: Option<ChronoDateTime>) {
        let l = vec![
            Lexeme::A,
            Lexeme::Month,
            Lexeme::After,
            Lexeme::October,
            Lexeme::Num(5),
        ];

        let today = now.map_or(Local::now().naive_local().date(), |now| now.date());
        let (date, t) = DateTime::parse(l.as_slice()).unwrap();
        let date = date
            .to_chrono(Local::now().naive_local().time(), now)
            .unwrap();

        assert_eq!(t, 5);
        assert_eq!(date.year(), today.year());
        assert_eq!(date.month(), 11);
        assert_eq!(date.day(), 5);
    }

    #[test_case(None; "default reference time")]
    #[test_case(Some(Local.with_ymd_and_hms(2021, 4, 30, 7, 15, 17).single().expect("literal date for test case").naive_local()); "past reference time")]
    fn test_year_after(now: Option<ChronoDateTime>) {
        let l = vec![
            Lexeme::A,
            Lexeme::Year,
            Lexeme::After,
            Lexeme::October,
            Lexeme::Num(5),
        ];

        let today = now.map_or(Local::now().naive_local().date(), |now| now.date());
        let (date, t) = DateTime::parse(l.as_slice()).unwrap();
        let date = date
            .to_chrono(Local::now().naive_local().time(), now)
            .unwrap();

        assert_eq!(t, 5);
        assert_eq!(date.year(), today.year() + 1);
        assert_eq!(date.month(), 10);
        assert_eq!(date.day(), 5);
    }

    #[test_case(None; "default reference time")]
    #[test_case(Some(Local.with_ymd_and_hms(2021, 4, 30, 7, 15, 17).single().expect("literal date for test case").naive_local()); "past reference time")]
    fn test_week_before(now: Option<ChronoDateTime>) {
        let l = vec![
            Lexeme::A,
            Lexeme::Week,
            Lexeme::Before,
            Lexeme::October,
            Lexeme::Num(15),
        ];

        let today = now.map_or(Local::now().naive_local().date(), |now| now.date());
        let (date, t) = DateTime::parse(l.as_slice()).unwrap();
        let date = date
            .to_chrono(Local::now().naive_local().time(), now)
            .unwrap();

        assert_eq!(t, 5);
        assert_eq!(date.year(), today.year());
        assert_eq!(date.month(), 10);
        assert_eq!(date.day(), 8);
    }

    #[test_case(None; "default reference time")]
    #[test_case(Some(Local.with_ymd_and_hms(2021, 4, 30, 7, 15, 17).single().expect("literal date for test case").naive_local()); "past reference time")]
    fn test_month_before(now: Option<ChronoDateTime>) {
        let l = vec![
            Lexeme::A,
            Lexeme::Month,
            Lexeme::Before,
            Lexeme::October,
            Lexeme::Num(5),
        ];

        let today = now.map_or(Local::now().naive_local().date(), |now| now.date());
        let (date, t) = DateTime::parse(l.as_slice()).unwrap();
        let date = date
            .to_chrono(Local::now().naive_local().time(), now)
            .unwrap();

        assert_eq!(t, 5);
        assert_eq!(date.year(), today.year());
        assert_eq!(date.month(), 9);
        assert_eq!(date.day(), 5);
    }

    #[test_case(None; "default reference time")]
    #[test_case(Some(Local.with_ymd_and_hms(2021, 4, 30, 7, 15, 17).single().expect("literal date for test case").naive_local()); "past reference time")]
    fn test_year_before(now: Option<ChronoDateTime>) {
        let l = vec![
            Lexeme::A,
            Lexeme::Year,
            Lexeme::Before,
            Lexeme::October,
            Lexeme::Num(5),
        ];

        let today = now.map_or(Local::now().naive_local().date(), |now| now.date());
        let (date, t) = DateTime::parse(l.as_slice()).unwrap();
        let date = date
            .to_chrono(Local::now().naive_local().time(), now)
            .unwrap();

        assert_eq!(t, 5);
        assert_eq!(date.year(), today.year() - 1);
        assert_eq!(date.month(), 10);
        assert_eq!(date.day(), 5);
    }

    #[test]
    fn test_month_before_to_leap_day() {
        let l = vec![
            Lexeme::Num(3),
            Lexeme::Month,
            Lexeme::Before,
            Lexeme::May,
            Lexeme::Num(31),
            Lexeme::Num(2024),
        ];

        let (date, t) = DateTime::parse(l.as_slice()).unwrap();
        let date = date
            .to_chrono(Local::now().naive_local().time(), None)
            .unwrap();

        assert_eq!(t, 6);
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 2);
        // 2024 is a leap year
        assert_eq!(date.day(), 29);
    }

    #[test]
    fn test_month_before_invalid_date() {
        let l = vec![
            Lexeme::Num(3),
            Lexeme::Month,
            Lexeme::Before,
            Lexeme::May,
            Lexeme::Num(31),
            Lexeme::Num(2023),
        ];

        let (date, t) = DateTime::parse(l.as_slice()).unwrap();
        let date = date
            .to_chrono(Local::now().naive_local().time(), None)
            .unwrap();

        assert_eq!(t, 6);
        assert_eq!(date.year(), 2023);
        assert_eq!(date.month(), 2);
        // 2024 is a leap year
        assert_eq!(date.day(), 28);
    }

    #[test_case(None; "default reference time")]
    #[test_case(Some(Local.with_ymd_and_hms(2021, 4, 30, 7, 15, 17).single().expect("literal date for test case").naive_local()); "past reference time")]
    fn test_next_week(now: Option<ChronoDateTime>) {
        let l = vec![Lexeme::Next, Lexeme::Week];

        let today = now.map_or(Local::now().naive_local(), |now| now);
        let (date, _) = DateTime::parse(l.as_slice()).unwrap();
        let date = date.to_chrono(today.time(), now).unwrap();

        assert_eq!(date, today + ChronoDuration::weeks(1));
    }

    #[test_case(None; "default reference time")]
    #[test_case(Some(Local.with_ymd_and_hms(2021, 4, 30, 7, 15, 17).single().expect("literal date for test case").naive_local()); "past reference time")]
    fn test_next_month(now: Option<ChronoDateTime>) {
        let l = vec![Lexeme::Next, Lexeme::Month];

        let today = now.map_or(Local::now().naive_local(), |now| now);

        let (date, _) = DateTime::parse(l.as_slice()).unwrap();
        let date = date.to_chrono(today.time(), now).unwrap();

        assert_eq!(
            date,
            today
                .checked_add_months(chrono::Months::new(1))
                .expect("Adding one month to current date shouldn't be the end of time.")
        );
    }

    #[test_case(None; "default reference time")]
    #[test_case(Some(Local.with_ymd_and_hms(2021, 4, 30, 7, 15, 17).single().expect("literal date for test case").naive_local()); "past reference time")]
    fn test_next_year(now: Option<ChronoDateTime>) {
        let l = vec![Lexeme::Next, Lexeme::Year];

        let today = now.map_or(Local::now().naive_local(), |now| now);
        let (date, _) = DateTime::parse(l.as_slice()).unwrap();
        let date = date.to_chrono(today.time(), now).unwrap();

        assert_eq!(
            date,
            today
                .with_year(today.year() + 1)
                .expect("Adding one year to current date shouldn't be the end of time.")
        );
    }

    #[test_case(None; "default reference time")]
    #[test_case(Some(Local.with_ymd_and_hms(2021, 4, 30, 7, 15, 17).single().expect("literal date for test case").naive_local()); "past reference time")]
    fn test_last_week(now: Option<ChronoDateTime>) {
        let l = vec![Lexeme::Last, Lexeme::Week];

        let today = now.map_or(Local::now().naive_local(), |now| now);
        let (date, _) = DateTime::parse(l.as_slice()).unwrap();
        let date = date.to_chrono(today.time(), now).unwrap();

        assert_eq!(date, today - ChronoDuration::weeks(1));
    }

    #[test_case(None; "default reference time")]
    #[test_case(Some(Local.with_ymd_and_hms(2021, 4, 30, 7, 15, 17).single().expect("literal date for test case").naive_local()); "past reference time")]
    fn test_last_month(now: Option<ChronoDateTime>) {
        let l = vec![Lexeme::Last, Lexeme::Month];

        let today = now.map_or(Local::now().naive_local(), |now| now);
        let (date, _) = DateTime::parse(l.as_slice()).unwrap();
        let date = date.to_chrono(today.time(), now).unwrap();

        assert_eq!(
            date,
            today
                .checked_sub_months(chrono::Months::new(1))
                .expect("Subtracting one month to current date shouldn't be the end of time.")
        );
    }

    #[test_case(None; "default reference time")]
    #[test_case(Some(Local.with_ymd_and_hms(2021, 4, 30, 7, 15, 17).single().expect("literal date for test case").naive_local()); "past reference time")]
    fn test_last_year(now: Option<ChronoDateTime>) {
        let l = vec![Lexeme::Last, Lexeme::Year];

        let today = now.map_or(Local::now().naive_local(), |now| now);
        let (date, _) = DateTime::parse(l.as_slice()).unwrap();
        let date = date.to_chrono(today.time(), now).unwrap();

        assert_eq!(
            date,
            today
                .with_year(today.year() - 1)
                .expect("Subtracting one year to current date shouldn't be the end of time.")
        );
    }

    #[test]
    fn test_month_literals_with_time_and_year() {
        use chrono::Timelike;

        let lexemes = vec![
            Lexeme::February,
            Lexeme::Num(16),
            Lexeme::Num(2022),
            Lexeme::Comma,
            Lexeme::Num(5),
            Lexeme::Colon,
            Lexeme::Num(27),
            Lexeme::PM,
        ];

        let (date, t) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date
            .to_chrono(Local::now().naive_local().time(), None)
            .unwrap();

        assert_eq!(t, 8);
        assert_eq!(date.year(), 2022);
        assert_eq!(date.month(), 2);
        assert_eq!(date.day(), 16);
        assert_eq!(date.hour(), 17);
        assert_eq!(date.minute(), 27);
    }

    #[test]
    fn test_slash_separated_date() {
        let lexemes = vec![
            Lexeme::Num(5),
            Lexeme::Slash,
            Lexeme::Num(12),
            Lexeme::Slash,
            Lexeme::Num(2023),
        ];

        let (date, t) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date
            .to_chrono(Local::now().naive_local().time(), None)
            .unwrap();

        assert_eq!(t, 5);
        assert_eq!(date.year(), 2023);
        assert_eq!(date.month(), 5);
        assert_eq!(date.day(), 12);
    }

    #[test]
    fn test_month_literals_with_time_and_no_year() {
        use chrono::Timelike;

        let lexemes = vec![
            Lexeme::February,
            Lexeme::Num(16),
            Lexeme::Comma,
            Lexeme::Num(5),
            Lexeme::Colon,
            Lexeme::Num(27),
            Lexeme::PM,
        ];
        let (date, t) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date
            .to_chrono(Local::now().naive_local().time(), None)
            .unwrap();
        let current_year = Local::now().naive_local().year();

        assert_eq!(t, 7);
        assert_eq!(date.year(), current_year);
        assert_eq!(date.month(), 2);
        assert_eq!(date.day(), 16);
        assert_eq!(date.hour(), 17);
        assert_eq!(date.minute(), 27);
    }

    #[test]
    fn test_slash_separated_invalid_month() {
        let lexemes = vec![
            Lexeme::Num(13),
            Lexeme::Slash,
            Lexeme::Num(12),
            Lexeme::Slash,
            Lexeme::Num(2023),
        ];
        let (date, _) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date.to_chrono(Local::now().naive_local().time(), None);

        assert!(date.is_err());
    }

    #[test]
    fn test_dash_separated_date() {
        let lexemes = vec![
            Lexeme::Num(5),
            Lexeme::Dash,
            Lexeme::Num(12),
            Lexeme::Dash,
            Lexeme::Num(2023),
        ];
        let (date, t) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date
            .to_chrono(Local::now().naive_local().time(), None)
            .unwrap();

        assert_eq!(t, 5);
        assert_eq!(date.year(), 2023);
        assert_eq!(date.month(), 5);
        assert_eq!(date.day(), 12);
    }

    #[test]
    fn test_dash_separated_invalid_month() {
        let lexemes = vec![
            Lexeme::Num(13),
            Lexeme::Dash,
            Lexeme::Num(12),
            Lexeme::Dash,
            Lexeme::Num(2023),
        ];
        let (date, _) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date.to_chrono(Local::now().naive_local().time(), None);

        assert!(date.is_err());
    }

    #[test]
    fn test_dot_separated_date() {
        let lexemes = vec![
            Lexeme::Num(19),
            Lexeme::Dot,
            Lexeme::Num(12),
            Lexeme::Dot,
            Lexeme::Num(2023),
        ];
        let (date, t) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date
            .to_chrono(Local::now().naive_local().time(), None)
            .unwrap();

        assert_eq!(t, 5);
        assert_eq!(date.year(), 2023);
        assert_eq!(date.month(), 12);
        assert_eq!(date.day(), 19);
    }

    #[test]
    fn test_dot_separated_date_invalid_month() {
        let lexemes = vec![
            Lexeme::Num(19),
            Lexeme::Dot,
            Lexeme::Num(13),
            Lexeme::Dot,
            Lexeme::Num(2023),
        ];
        let (date, _) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date.to_chrono(Local::now().naive_local().time(), None);

        assert!(date.is_err());
    }
}
