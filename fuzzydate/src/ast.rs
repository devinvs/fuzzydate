use chrono::{
    DateTime as ChronoDateTime, Datelike, Duration as ChronoDuration, NaiveDate as ChronoDate,
    NaiveDateTime, NaiveTime as ChronoTime, TimeZone, Weekday as ChronoWeekday,
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

            if l.get(tokens) == Some(&Lexeme::At) {
                tokens += 1;
            }

            // this needs to match before time, because time will match a bare number and duration
            // can be <num> <unit>, so we'll partially parse a time out of a duration and fail.
            if let Some((dur, t)) = Duration::parse(&l[tokens..]) {
                tokens += t;

                if Some(&Lexeme::After) == l.get(tokens) {
                    tokens += 1;

                    if let Some((time, t)) = Time::parse(&l[tokens..]) {
                        tokens += t;
                        let datetime = Self::DateTime(date, time);
                        return Some((Self::After(dur, Box::new(datetime)), tokens));
                    }
                } else if Some(&Lexeme::Before) == l.get(tokens) {
                    tokens += 1;

                    if let Some((time, t)) = Time::parse(&l[tokens..]) {
                        tokens += t;
                        let datetime = Self::DateTime(date, time);
                        return Some((Self::Before(dur, Box::new(datetime)), tokens));
                    }
                }
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

            if l.get(tokens) == Some(&Lexeme::On) {
                tokens += 1;
            }

            if let Some((date, t)) = Date::parse(&l[tokens..]) {
                tokens += t;
                return Some((Self::TimeDate(time, date), tokens));
            }
        }

        None
    }

    /// Convert a parsed DateTime to chrono's DateTime
    pub fn to_chrono<Tz: TimeZone>(
        &self,
        now: ChronoDateTime<Tz>,
    ) -> Result<ChronoDateTime<Tz>, crate::Error> {
        Ok(match self {
            DateTime::Now => now,
            DateTime::DateTime(date, time) => {
                let date = date.to_chrono(now.to_owned())?;
                let time = time.to_chrono(now.to_owned())?;

                // TODO: how to handle DST?
                NaiveDateTime::new(date, time)
                    .and_local_timezone(now.timezone())
                    .earliest()
                    .ok_or(crate::Error::ParseError)?
            }
            DateTime::TimeDate(time, date) => {
                let date = date.to_chrono(now.to_owned())?;
                let time = time.to_chrono(now.to_owned())?;

                // TODO: how to handle DST?
                NaiveDateTime::new(date, time)
                    .and_local_timezone(now.timezone())
                    .earliest()
                    .ok_or(crate::Error::ParseError)?
            }
            DateTime::After(dur, date) => {
                let date = date.to_chrono(now)?;
                dur.after(date)
            }
            DateTime::Before(dur, date) => {
                let date = date.to_chrono(now)?;
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

    fn to_chrono<Tz: TimeZone>(&self, now: ChronoDateTime<Tz>) -> Result<ChronoDate, crate::Error> {
        let mut today = now.date_naive();

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
                // TODO: match

                let date;

                if relspec == &RelativeSpecifier::Next {
                    date = Duration::Specific(1, unit.to_owned())
                        .after(now)
                        .date_naive();
                } else if relspec == &RelativeSpecifier::Last {
                    date = Duration::Specific(1, unit.to_owned())
                        .before(now)
                        .date_naive();
                } else {
                    unreachable!();
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

        if let Some(&Lexeme::Midnight) = l.get(tokens) {
            tokens += 1;
            return Some((Time::HourMin(0, 0), tokens));
        }

        if let Some(&Lexeme::Noon) = l.get(tokens) {
            tokens += 1;
            return Some((Time::HourMin(12, 0), tokens));
        }

        if let Some((hour, t)) = Num::parse(&l[tokens..]) {
            tokens += t;
            let mut minute = 0;
            if l.get(tokens) == Some(&Lexeme::Colon) {
                tokens += 1;

                if let Some((min, t)) = Num::parse(&l[tokens..]) {
                    tokens += t;
                    minute = min;
                }
            }
            if let Some(&Lexeme::AM) = l.get(tokens) {
                tokens += 1;
                return Some((Time::HourMinAM(hour, minute), tokens));
            } else if let Some(&Lexeme::PM) = l.get(tokens) {
                tokens += 1;
                return Some((Time::HourMinPM(hour, minute), tokens));
            } else {
                return Some((Time::HourMin(hour, minute), tokens));
            }
        }

        tokens = 0;
        Some((Self::Empty, tokens))
    }

    fn to_chrono<Tz: TimeZone>(&self, now: ChronoDateTime<Tz>) -> Result<ChronoTime, crate::Error> {
        match *self {
            Time::Empty => Ok(now.time()),
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
    // TODO: combine direction with duration rather than datetime
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

    fn convertible(&self) -> bool {
        if let Duration::Concat(dur1, dur2) = self {
            return dur1.convertible() && dur2.convertible();
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

    fn after<Tz: TimeZone>(&self, date: ChronoDateTime<Tz>) -> ChronoDateTime<Tz> {
        if let Duration::Concat(dur1, dur2) = self {
            return dur2.after(dur1.after(date));
        }

        if self.convertible() {
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

    fn before<Tz: TimeZone>(&self, date: ChronoDateTime<Tz>) -> ChronoDateTime<Tz> {
        if let Duration::Concat(dur1, dur2) = self {
            return dur2.before(dur1.before(date));
        }

        if self.convertible() {
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
    // TODO: split implementations and tests into separate files

    use chrono::{Local, TimeZone};

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
        let (parsed, t) = DateTime::parse(lexemes.as_slice()).unwrap();
        let result = parsed.to_chrono(Local::now()).unwrap();

        assert_eq!(t, 7);
        assert_eq!(result.year(), 2022);
        assert_eq!(result.month(), 2);
        assert_eq!(result.day(), 16);
        assert_eq!(result.hour(), 17);
        assert_eq!(result.minute(), 27);
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
            Lexeme::Tomorrow,
            Lexeme::Comma,
            Lexeme::Num(5),
            Lexeme::Colon,
            Lexeme::Num(20),
        ];

        use chrono::naive::Days;
        let now = Local
            .with_ymd_and_hms(2021, 4, 30, 7, 15, 17)
            .single()
            .expect("literal date for test case");
        let today = now.date_naive();
        let real_date = today + Days::new(7 - 2 + 1 + 1);

        let (parsed, t) = DateTime::parse(lexemes.as_slice()).unwrap();
        let result = parsed.to_chrono(now).unwrap();

        assert_eq!(t, 14);
        assert_eq!(result.year(), real_date.year());
        assert_eq!(result.month(), real_date.month());
        assert_eq!(result.day(), real_date.day());
    }

    #[test]
    fn test_datetime_now() {
        use chrono::Timelike;

        let lexemes = vec![Lexeme::Now];
        let (parsed, t) = DateTime::parse(lexemes.as_slice()).unwrap();
        let now = Local
            .with_ymd_and_hms(2021, 4, 30, 7, 15, 17)
            .single()
            .expect("literal date for test case");
        let result = parsed.to_chrono(now).unwrap();

        assert_eq!(t, 1);
        assert_eq!(result.year(), now.year());
        assert_eq!(result.month(), now.month());
        assert_eq!(result.day(), now.day());
        assert_eq!(result.hour(), now.hour());
        assert_eq!(result.minute(), now.minute());
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

    #[test]
    fn test_datetime_ago() {
        let lexemes = vec![Lexeme::A, Lexeme::Day, Lexeme::Ago];
        let now = Local
            .with_ymd_and_hms(2021, 4, 30, 7, 15, 17)
            .single()
            .expect("literal date for test case");
        let (parsed, t) = DateTime::parse(lexemes.as_slice()).unwrap();
        let result = parsed.to_chrono(now).unwrap();

        let today = now.date_naive();
        assert_eq!(t, 3);
        assert_eq!(result.year(), today.year());
        assert_eq!(result.month(), today.month());
        assert_eq!(result.day(), today.day() - 1);
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

    #[test]
    fn test_article_before() {
        let now = Local
            .with_ymd_and_hms(2021, 4, 30, 7, 15, 17)
            .single()
            .expect("literal date for test case");
        let (parsed, t) =
            DateTime::parse(&[Lexeme::A, Lexeme::Day, Lexeme::Before, Lexeme::Today]).unwrap();
        let result = parsed.to_chrono(now).unwrap();

        let today = now.date_naive();
        assert_eq!(t, 4);
        assert_eq!(result.year(), today.year());
        assert_eq!(result.month(), today.month());
        assert_eq!(result.day(), today.day() - 1);
    }

    #[test]
    fn test_after_december() {
        let l = vec![
            Lexeme::A,
            Lexeme::Month,
            Lexeme::After,
            Lexeme::December,
            Lexeme::Num(5),
        ];
        let now = Local
            .with_ymd_and_hms(2021, 4, 30, 7, 15, 17)
            .single()
            .expect("literal date for test case");

        let (date, t) = DateTime::parse(l.as_slice()).unwrap();
        let date = date.to_chrono(now).unwrap();

        assert_eq!(t, 5);
        assert_eq!(date.year(), now.year() + 1);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 5);
    }

    #[test]
    fn test_month_before_january() {
        let l = vec![
            Lexeme::A,
            Lexeme::Month,
            Lexeme::Before,
            Lexeme::January,
            Lexeme::Num(5),
        ];

        let now = Local
            .with_ymd_and_hms(2021, 4, 30, 7, 15, 17)
            .single()
            .expect("literal date for test case");

        let (date, t) = DateTime::parse(l.as_slice()).unwrap();
        let date = date.to_chrono(now).unwrap();

        assert_eq!(t, 5);
        assert_eq!(date.year(), now.year() - 1);
        assert_eq!(date.month(), 12);
        assert_eq!(date.day(), 5);
    }

    #[test]
    fn test_month_after() {
        let l = vec![
            Lexeme::A,
            Lexeme::Month,
            Lexeme::After,
            Lexeme::October,
            Lexeme::Num(5),
        ];
        let now = Local
            .with_ymd_and_hms(2021, 4, 30, 7, 15, 17)
            .single()
            .expect("literal date for test case");

        let (date, t) = DateTime::parse(l.as_slice()).unwrap();
        let date = date.to_chrono(now).unwrap();

        assert_eq!(t, 5);
        assert_eq!(date.year(), now.year());
        assert_eq!(date.month(), 11);
        assert_eq!(date.day(), 5);
    }

    #[test]
    fn test_year_after() {
        let l = vec![
            Lexeme::A,
            Lexeme::Year,
            Lexeme::After,
            Lexeme::October,
            Lexeme::Num(5),
        ];

        let now = Local
            .with_ymd_and_hms(2021, 4, 30, 7, 15, 17)
            .single()
            .expect("literal date for test case");

        let (date, t) = DateTime::parse(l.as_slice()).unwrap();
        let date = date.to_chrono(now).unwrap();

        assert_eq!(t, 5);
        assert_eq!(date.year(), now.year() + 1);
        assert_eq!(date.month(), 10);
        assert_eq!(date.day(), 5);
    }

    #[test]
    fn test_month_before() {
        let l = vec![
            Lexeme::A,
            Lexeme::Month,
            Lexeme::Before,
            Lexeme::October,
            Lexeme::Num(5),
        ];

        let now = Local
            .with_ymd_and_hms(2021, 4, 30, 7, 15, 17)
            .single()
            .expect("literal date for test case");
        let (date, t) = DateTime::parse(l.as_slice()).unwrap();
        let date = date.to_chrono(now).unwrap();

        assert_eq!(t, 5);
        assert_eq!(date.year(), now.year());
        assert_eq!(date.month(), 9);
        assert_eq!(date.day(), 5);
    }

    #[test]
    fn test_year_before() {
        let l = vec![
            Lexeme::A,
            Lexeme::Year,
            Lexeme::Before,
            Lexeme::October,
            Lexeme::Num(5),
        ];
        let now = Local
            .with_ymd_and_hms(2021, 4, 30, 7, 15, 17)
            .single()
            .expect("literal date for test case");
        let (date, t) = DateTime::parse(l.as_slice()).unwrap();
        let date = date.to_chrono(now).unwrap();

        assert_eq!(t, 5);
        assert_eq!(date.year(), now.year() - 1);
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

        let now = Local::now();
        let (date, t) = DateTime::parse(l.as_slice()).unwrap();
        let date = date.to_chrono(now).unwrap();

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

        let now = Local::now();
        let (date, t) = DateTime::parse(l.as_slice()).unwrap();
        let date = date.to_chrono(now).unwrap();

        assert_eq!(t, 6);
        assert_eq!(date.year(), 2023);
        assert_eq!(date.month(), 2);
        // 2024 is a leap year
        assert_eq!(date.day(), 28);
    }

    #[test]
    fn test_next_week() {
        let l = vec![Lexeme::Next, Lexeme::Week];

        let now = Local::now();
        let (date, _) = DateTime::parse(l.as_slice()).unwrap();
        let date = date.to_chrono(now).unwrap();

        assert_eq!(date, now + ChronoDuration::weeks(1));
    }

    #[test]
    fn test_next_month() {
        let l = vec![Lexeme::Next, Lexeme::Month];

        let now = Local::now();
        let (date, _) = DateTime::parse(l.as_slice()).unwrap();
        let date = date.to_chrono(now).unwrap();

        assert_eq!(
            date,
            now.checked_add_months(chrono::Months::new(1))
                .expect("Adding one month to current date shouldn't be the end of time.")
        );
    }

    #[test]
    fn test_next_year() {
        let l = vec![Lexeme::Next, Lexeme::Year];

        let now = Local::now();
        let (date, _) = DateTime::parse(l.as_slice()).unwrap();
        let date = date.to_chrono(now).unwrap();

        assert_eq!(
            date,
            now.with_year(now.year() + 1)
                .expect("Adding one year to current date shouldn't be the end of time.")
        );
    }

    #[test]
    fn test_last_week() {
        let l = vec![Lexeme::Last, Lexeme::Week];

        let now = Local::now();
        let (date, _) = DateTime::parse(l.as_slice()).unwrap();
        let date = date.to_chrono(now).unwrap();

        assert_eq!(date, now - ChronoDuration::weeks(1));
    }

    #[test]
    fn test_last_month() {
        let l = vec![Lexeme::Last, Lexeme::Month];

        let now = Local::now();
        let (date, _) = DateTime::parse(l.as_slice()).unwrap();
        let date = date.to_chrono(now).unwrap();

        assert_eq!(
            date,
            now.checked_sub_months(chrono::Months::new(1))
                .expect("Subtracting one month to current date shouldn't be the end of time.")
        );
    }

    #[test]
    fn test_last_year() {
        let l = vec![Lexeme::Last, Lexeme::Year];

        let now = Local::now();
        let (date, _) = DateTime::parse(l.as_slice()).unwrap();
        let date = date.to_chrono(now).unwrap();

        assert_eq!(
            date,
            now.with_year(now.year() - 1)
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

        let now = Local::now();
        let (date, t) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date.to_chrono(now).unwrap();

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

        let now = Local::now();
        let (date, t) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date.to_chrono(now).unwrap();

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

        let now = Local::now();
        let (date, t) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date.to_chrono(now).unwrap();
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

        let now = Local::now();
        let (date, _) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date.to_chrono(now);

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
        let now = Local::now();
        let (date, t) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date.to_chrono(now).unwrap();

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
        let now = Local::now();
        let (date, _) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date.to_chrono(now);

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
        let now = Local::now();
        let (date, t) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date.to_chrono(now).unwrap();

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
        let now = Local::now();
        let (date, _) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date.to_chrono(now);

        assert!(date.is_err());
    }

    // TODO: test dst transitions, timezone awareness, datetime with duration sandwich
}
