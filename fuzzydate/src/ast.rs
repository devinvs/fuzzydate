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
    /// Time before date
    TimeDate(Time, Date),
    /// Duration after a datetime
    After(Duration, Box<DateTime>),
    /// Duration before a datetime
    Before(Duration, Box<DateTime>),
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

        if let Some((date_expr, t)) = Date::parse(&l[tokens..]) {
            tokens += t;

            if l.get(tokens) == Some(&Lexeme::Comma) || l.get(tokens) == Some(&Lexeme::At) {
                tokens += 1;
            }

            if let Some((time, t)) = Time::parse(&l[tokens..]) {
                tokens += t;
                return Some((Self::DateTime(date_expr, time), tokens));
            }

            return Some((Self::DateTime(date_expr, Time::Empty), tokens));
        }

        // Date also accepts a leading duration and should take precedence. For example,
        // "3 days ago" is a date specified by duration, not a duration modifying a datetime
        if let Some((dur, t)) = Duration::parse(&l[tokens..]) {
            tokens += t;

            if Some(&Lexeme::After) == l.get(tokens) {
                tokens += 1;

                if let Some((datetime, t)) = DateTime::parse(&l[tokens..]) {
                    tokens += t;
                    return Some((Self::After(dur, Box::new(datetime)), tokens));
                }

                return None;
            }

            if Some(&Lexeme::From) == l.get(tokens) {
                tokens += 1;

                if let Some((datetime, t)) = DateTime::parse(&l[tokens..]) {
                    tokens += t;
                    return Some((Self::After(dur, Box::new(datetime)), tokens));
                }

                return None;
            }

            if Some(&Lexeme::Before) == l.get(tokens) {
                tokens += 1;

                if let Some((datetime, t)) = DateTime::parse(&l[tokens..]) {
                    tokens += t;
                    return Some((Self::Before(dur, Box::new(datetime)), tokens));
                }

                return None;
            }

            if Some(&Lexeme::Ago) == l.get(tokens) {
                tokens += 1;
                return Some((Self::Before(dur, Box::new(Self::Now)), tokens));
            }

            return None;
        }

        // time binds really eagerly. A bare number is a valid time, but can also be the start of a
        // date or duration expression, so we need to check time last
        if let Some((time, t)) = Time::parse(&l[tokens..]) {
            tokens += t;

            if l.get(tokens) == Some(&Lexeme::Comma) || l.get(tokens) == Some(&Lexeme::On) {
                tokens += 1;
            }

            if let Some((date_expr, t)) = Date::parse(&l[tokens..]) {
                tokens += t;
                return Some((Self::TimeDate(time, date_expr), tokens));
            }

            return Some((Self::TimeDate(time, Date::Empty), tokens));
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

                NaiveDateTime::new(date, time)
                    .and_local_timezone(now.timezone())
                    .earliest()
                    .ok_or(crate::Error::ParseError)?
            }
            DateTime::TimeDate(time, date) => {
                let date = date.to_chrono(now.to_owned())?;
                let time = time.to_chrono(now.to_owned())?;

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
    Today,
    Tomorrow,
    Yesterday,
    Before(Duration, Box<Date>),
    After(Duration, Box<Date>),
    UnitRelative(RelativeSpecifier, Unit),
    Relative(RelativeSpecifier, Weekday),
    Weekday(Weekday),
    Empty,
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
            }

            return Some((Self::MonthDay(month, day), tokens));
        }

        // TODO: year month day
        tokens = 0;
        if let Some((num1, t)) = Num::parse(&l[tokens..]) {
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

        let mut tokens = 0;

        if let Some((duration, t)) = Duration::parse(l) {
            tokens += t;

            if duration.is_sub_daily() {
                return None;
            }

            if l.get(tokens) == Some(&Lexeme::Ago) {
                tokens += 1;
                return Some((Self::Before(duration, Box::new(Self::Today)), tokens));
            }

            if l.get(tokens) == Some(&Lexeme::After) {
                tokens += 1;
                if let Some((date, t)) = Self::parse(&l[tokens..]) {
                    tokens += t;

                    return Some((Self::After(duration, Box::new(date)), tokens));
                }

                return None;
            }

            if l.get(tokens) == Some(&Lexeme::From) {
                tokens += 1;
                if let Some((date, t)) = Self::parse(&l[tokens..]) {
                    tokens += t;

                    return Some((Self::After(duration, Box::new(date)), tokens));
                }

                return None;
            }

            return None;
        }

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

            return None;
        }

        if let Some((weekday, t)) = Weekday::parse(&l[tokens..]) {
            tokens += t;
            return Some((Self::Weekday(weekday), tokens));
        }

        None
    }

    fn to_chrono<Tz: TimeZone>(&self, now: ChronoDateTime<Tz>) -> Result<ChronoDate, crate::Error> {
        let today = now.date_naive();
        Ok(match self {
            Self::Today => today,
            Self::Yesterday => today - ChronoDuration::days(1),
            Self::Tomorrow => today + ChronoDuration::days(1),
            Self::MonthNumDay(month, day) => ChronoDate::from_ymd_opt(today.year(), *month, *day)
                .ok_or(crate::Error::InvalidDate(format!(
                "Invalid month-day: {month}-{day}"
            )))?,
            Self::MonthNumDayYear(month, day, year) => {
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
            Self::MonthDay(month, day) => {
                let month = *month as u32;
                ChronoDate::from_ymd_opt(today.year(), month, *day).ok_or(
                    crate::Error::InvalidDate(format!("Invalid month-day: {month}-{day}")),
                )?
            }
            Self::MonthDayYear(month, day, year) => {
                ChronoDate::from_ymd_opt(*year as i32, *month as u32, *day).ok_or(
                    crate::Error::InvalidDate(format!(
                        "Invalid year-month-day: {}-{}-{}",
                        *year, *month as u32, *day
                    )),
                )?
            }
            Self::Before(dur, date) => dur.before_date(date.to_chrono(now)?)?,
            Self::After(dur, date) => dur.after_date(date.to_chrono(now)?)?,
            Self::UnitRelative(spec, unit) => {
                match spec {
                    RelativeSpecifier::Next => {
                        Duration::Specific(1, unit.to_owned()).after_date(now.date_naive())?
                    }
                    RelativeSpecifier::Last => {
                        Duration::Specific(1, unit.to_owned()).before_date(now.date_naive())?
                    }
                    RelativeSpecifier::This => {
                        // This would be nonsense as far as I can tell. An example would
                        // be "2pm this month"
                        return Err(crate::Error::ParseError);
                    }
                }
            }
            Self::Relative(spec, day) => {
                let day = day.to_chrono();
                let mut today = now.date_naive();
                let this_week = today.iso_week();

                match spec {
                    RelativeSpecifier::Next => {
                        while today.iso_week() == this_week {
                            today += ChronoDuration::days(1);
                        }

                        while today.weekday() != day {
                            today += ChronoDuration::days(1);
                        }
                    }
                    RelativeSpecifier::Last => {
                        while today.iso_week() == this_week {
                            today -= ChronoDuration::days(1);
                        }
                        while today.weekday() != day {
                            today -= ChronoDuration::days(1);
                        }
                    }
                    RelativeSpecifier::This => {
                        while today.iso_week() == this_week {
                            today -= ChronoDuration::days(1);
                        }
                        today += ChronoDuration::days(1);

                        while today.weekday() != day {
                            today += ChronoDuration::days(1);
                        }
                    }
                }

                today
            }
            Self::Weekday(day) => {
                let day = day.to_chrono();
                let mut today = now.date_naive();

                while today.weekday() != day {
                    today += ChronoDuration::days(1);
                }

                today
            }
            Self::Empty => now.date_naive(),
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
        let res = match l.first() {
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
        let res = match l.first() {
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
        let res = match l.first() {
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

        None
    }

    fn to_chrono<Tz: TimeZone>(&self, now: ChronoDateTime<Tz>) -> Result<ChronoTime, crate::Error> {
        match *self {
            Self::Empty => Ok(now.time()),
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
        match l.first() {
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

    fn after_date(&self, date: ChronoDate) -> Result<ChronoDate, crate::Error> {
        if self.is_sub_daily() {
            // TODO: better error
            return Err(crate::Error::ParseError);
        }

        if let Duration::Concat(dur1, dur2) = self {
            return dur2.after_date(dur1.after_date(date)?);
        }

        if self.convertible() {
            Ok(date + self.to_chrono())
        } else {
            match self.unit() {
                Unit::Month => date
                    .checked_add_months(chrono::Months::new(self.num()))
                    .ok_or(crate::Error::ParseError),
                Unit::Year => date
                    .with_year(date.year() + self.num() as i32)
                    .ok_or(crate::Error::ParseError),
                _ => unreachable!(),
            }
        }
    }

    fn before_date(&self, date: ChronoDate) -> Result<ChronoDate, crate::Error> {
        if self.is_sub_daily() {
            // TODO: better error
            return Err(crate::Error::ParseError);
        }

        if let Duration::Concat(dur1, dur2) = self {
            return dur2.before_date(dur1.before_date(date)?);
        }

        if self.convertible() {
            Ok(date - self.to_chrono())
        } else {
            match self.unit() {
                Unit::Month => date
                    .checked_sub_months(chrono::Months::new(self.num()))
                    .ok_or(crate::Error::ParseError),
                Unit::Year => date
                    .with_year(date.year() - self.num() as i32)
                    .ok_or(crate::Error::ParseError),
                _ => unreachable!(),
            }
        }
    }

    fn is_sub_daily(&self) -> bool {
        match self {
            Self::Article(unit) | Self::Specific(_, unit) => match unit {
                Unit::Day | Unit::Week | Unit::Month | Unit::Year => false,
                Unit::Hour | Unit::Minute => true,
            },
            Self::Concat(a, b) => a.is_sub_daily() || b.is_sub_daily(),
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
        match l.first() {
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
        let mut res = match l.first() {
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
            if let Some(Lexeme::Num(n)) = l.first() {
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
        let mut res = match l.first() {
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
            if let Some(Lexeme::Num(n)) = l.first() {
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
        match l.first() {
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
        match l.first() {
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

    use chrono::{Local, Months, TimeZone};

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
    fn test_noon_date_time() {
        use chrono::Timelike;

        let lexemes = vec![
            Lexeme::February,
            Lexeme::Num(16),
            Lexeme::Num(2022),
            Lexeme::Noon,
        ];
        let (date, t) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date.to_chrono(Local::now()).unwrap();

        assert_eq!(t, 4);
        assert_eq!(date.year(), 2022);
        assert_eq!(date.month(), 2);
        assert_eq!(date.day(), 16);
        assert_eq!(date.hour(), 12);
        assert_eq!(date.minute(), 0);
    }

    #[test]
    fn test_midnight_date_time() {
        use chrono::Timelike;

        let lexemes = vec![
            Lexeme::February,
            Lexeme::Num(16),
            Lexeme::Num(2022),
            Lexeme::Midnight,
        ];
        let (date, t) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date.to_chrono(Local::now()).unwrap();

        assert_eq!(t, 4);
        assert_eq!(date.year(), 2022);
        assert_eq!(date.month(), 2);
        assert_eq!(date.day(), 16);
        assert_eq!(date.hour(), 0);
        assert_eq!(date.minute(), 0);
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
    fn test_next_weekday_from_week_start() {
        let l = vec![Lexeme::Next, Lexeme::Monday];

        // 12 Apr 2021 is a Monday
        let now = Local
            .with_ymd_and_hms(2021, 4, 12, 7, 15, 17)
            .single()
            .expect("literal datetime for test case");
        let (date, _) = DateTime::parse(l.as_slice()).unwrap();

        let expected = Local
            .with_ymd_and_hms(2021, 4, 19, 7, 15, 17)
            .single()
            .expect("literal datetime for test case");
        let date = date.to_chrono(now).unwrap();

        assert_eq!(date, expected);
    }

    #[test]
    fn test_next_weekday_from_week_end() {
        let l = vec![Lexeme::Next, Lexeme::Monday];

        // 18 Apr 2021 is a Sunday
        let now = Local
            .with_ymd_and_hms(2021, 4, 18, 7, 15, 17)
            .single()
            .expect("literal datetime for test case");
        let (date, _) = DateTime::parse(l.as_slice()).unwrap();

        let expected = Local
            .with_ymd_and_hms(2021, 4, 19, 7, 15, 17)
            .single()
            .expect("literal datetime for test case");
        let date = date.to_chrono(now).unwrap();

        assert_eq!(date, expected);
    }

    #[test]
    fn test_last_weekday_from_week_start() {
        let l = vec![Lexeme::Last, Lexeme::Monday];

        // 12 Apr 2021 is a Monday
        let now = Local
            .with_ymd_and_hms(2021, 4, 12, 7, 15, 17)
            .single()
            .expect("literal datetime for test case");
        let (date, _) = DateTime::parse(l.as_slice()).unwrap();

        let expected = Local
            .with_ymd_and_hms(2021, 4, 5, 7, 15, 17)
            .single()
            .expect("literal datetime for test case");
        let date = date.to_chrono(now).unwrap();

        assert_eq!(date, expected);
    }

    #[test]
    fn test_last_weekday_from_week_end() {
        let l = vec![Lexeme::Last, Lexeme::Monday];

        // 18 Apr 2021 is a Sunday
        let now = Local
            .with_ymd_and_hms(2021, 4, 18, 7, 15, 17)
            .single()
            .expect("literal datetime for test case");
        let (date, _) = DateTime::parse(l.as_slice()).unwrap();

        let expected = Local
            .with_ymd_and_hms(2021, 4, 5, 7, 15, 17)
            .single()
            .expect("literal datetime for test case");
        let date = date.to_chrono(now).unwrap();

        assert_eq!(date, expected);
    }

    #[test]
    fn test_this_weekday_from_week_start() {
        let l = vec![Lexeme::This, Lexeme::Monday];

        // 12 Apr 2021 is a Monday
        let now = Local
            .with_ymd_and_hms(2021, 4, 12, 7, 15, 17)
            .single()
            .expect("literal datetime for test case");
        let (date, _) = DateTime::parse(l.as_slice()).unwrap();

        let expected = Local
            .with_ymd_and_hms(2021, 4, 12, 7, 15, 17)
            .single()
            .expect("literal datetime for test case");
        let date = date.to_chrono(now).unwrap();

        assert_eq!(date, expected);
    }

    #[test]
    fn test_this_weekday_from_week_end() {
        let l = vec![Lexeme::This, Lexeme::Monday];

        // 18 Apr 2021 is a Sunday
        let now = Local
            .with_ymd_and_hms(2021, 4, 18, 7, 15, 17)
            .single()
            .expect("literal datetime for test case");
        let (date, _) = DateTime::parse(l.as_slice()).unwrap();

        let expected = Local
            .with_ymd_and_hms(2021, 4, 12, 7, 15, 17)
            .single()
            .expect("literal datetime for test case");
        let date = date.to_chrono(now).unwrap();

        assert_eq!(date, expected);
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

    #[test]
    fn test_date_day_ago() {
        let lexemes = vec![Lexeme::Num(3), Lexeme::Day, Lexeme::Ago];
        let now = Local
            .with_ymd_and_hms(2021, 4, 30, 7, 15, 17)
            .single()
            .expect("literal date for test case");
        let (date, tokens) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date.to_chrono(now);

        let date = date.unwrap();

        assert_eq!(tokens, 3);

        assert_eq!(
            now.date_naive() - date.date_naive(),
            ChronoDuration::days(3)
        );
        assert_eq!(now.time(), date.time());
    }

    #[test]
    fn test_date_day_after() {
        let lexemes = vec![Lexeme::Num(3), Lexeme::Day, Lexeme::After, Lexeme::Now];
        let now = Local
            .with_ymd_and_hms(2021, 4, 30, 7, 15, 17)
            .single()
            .expect("literal date for test case");

        let (date, tokens) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date.to_chrono(now);

        let date = date.unwrap();

        assert_eq!(tokens, 4);

        assert_eq!(
            date.date_naive() - now.date_naive(),
            ChronoDuration::days(3)
        );
        assert_eq!(now.time(), date.time());
    }

    #[test]
    fn test_date_day_before() {
        let lexemes = vec![Lexeme::Num(3), Lexeme::Day, Lexeme::Before, Lexeme::Now];
        let now = Local
            .with_ymd_and_hms(2021, 4, 30, 7, 15, 17)
            .single()
            .expect("literal date for test case");

        let (date, tokens) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date.to_chrono(now);

        let date = date.unwrap();

        assert_eq!(tokens, 4);

        assert_eq!(
            now.date_naive() - ChronoDuration::days(3),
            date.date_naive()
        );

        assert_eq!(now.time(), date.time());
    }

    #[test]
    fn test_date_month_after() {
        let lexemes = vec![Lexeme::Num(3), Lexeme::Month, Lexeme::After, Lexeme::Now];
        let now = Local
            .with_ymd_and_hms(2021, 4, 30, 7, 15, 17)
            .single()
            .expect("literal date for test case");

        let (date, tokens) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date.to_chrono(now);

        let date = date.unwrap();

        assert_eq!(tokens, 4);

        assert_eq!(
            now.date_naive().checked_add_months(Months::new(3)).unwrap(),
            date.date_naive()
        );
        assert_eq!(now.time(), date.time());
    }

    #[test]
    fn test_date_month_before() {
        let lexemes = vec![Lexeme::Num(3), Lexeme::Month, Lexeme::Before, Lexeme::Now];
        let now = Local
            .with_ymd_and_hms(2021, 4, 30, 7, 15, 17)
            .single()
            .expect("literal date for test case");

        let (date, tokens) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date.to_chrono(now);

        let date = date.unwrap();

        assert_eq!(tokens, 4);

        assert_eq!(
            now.date_naive().checked_sub_months(Months::new(3)).unwrap(),
            date.date_naive()
        );
        assert_eq!(now.time(), date.time());
    }

    #[test]
    fn test_date_minute_after() {
        let lexemes = vec![
            Lexeme::Num(3),
            Lexeme::Minute,
            Lexeme::After,
            Lexeme::Yesterday,
        ];
        // Duration with sub-daily unit should not be parsed as a Date expression
        // so Date::parse should return None.
        assert!(Date::parse(lexemes.as_slice()).is_none());
    }

    #[test]
    fn test_date_minute_before() {
        let lexemes = vec![
            Lexeme::Num(3),
            Lexeme::Minute,
            Lexeme::Before,
            Lexeme::Yesterday,
        ];
        // Duration with sub-daily unit should not be parsed as a Date expression
        // so Date::parse should return None.
        assert!(Date::parse(lexemes.as_slice()).is_none());
    }

    #[test]
    fn test_march_dst_transition() {
        // March DST transition in America/New_York creates a non-existent local time
        use chrono_tz::America::New_York;

        let lexemes = vec![
            Lexeme::March,
            Lexeme::Num(14),
            Lexeme::Num(2021),
            Lexeme::Num(2),
            Lexeme::Colon,
            Lexeme::Num(30),
        ];

        let now = New_York.with_ymd_and_hms(2021, 3, 14, 12, 0, 0).unwrap();
        let (parsed, _) = DateTime::parse(lexemes.as_slice()).unwrap();
        let res = parsed.to_chrono(now);

        // 2:30 on this date in New York is a nonexistent local time -> conversion fails
        assert!(res.is_err());
    }

    #[test]
    fn test_november_dst_transition() {
        // November DST transition in America/New_York repeats the 1 AM hour.
        // We should pick the earlier of the two.
        use chrono_tz::America::New_York;

        let lexemes = vec![
            Lexeme::November,
            Lexeme::Num(7),
            Lexeme::Num(2021),
            Lexeme::Num(1),
            Lexeme::Colon,
            Lexeme::Num(30),
            Lexeme::AM,
        ];

        let now = New_York.with_ymd_and_hms(2021, 11, 7, 12, 0, 0).unwrap();
        let (parsed, _) = DateTime::parse(lexemes.as_slice()).unwrap();
        let res = parsed.to_chrono(now).unwrap();

        // Earlier 1:30 on this date is during DST (UTC-4) => offset (local - utc) should be -14400
        assert_eq!(
            res.naive_local()
                .signed_duration_since(res.naive_utc())
                .num_seconds(),
            -4 * 3600
        );
    }

    #[test]
    fn test_return_correct_timezone_dst() {
        // Adding an hour across the spring-forward DST boundary should succeed for
        // convertible durations (hours) and result in the correct local time in the
        // timezone of `now`
        use chrono::Timelike;
        use chrono_tz::America::New_York;

        let lexemes = vec![
            Lexeme::A,
            Lexeme::Hour,
            Lexeme::After,
            Lexeme::March,
            Lexeme::Num(14),
            Lexeme::Num(2021),
            Lexeme::Num(1),
            Lexeme::Colon,
            Lexeme::Num(30),
        ];

        let now = New_York.with_ymd_and_hms(2021, 3, 14, 12, 0, 0).unwrap();
        let (parsed, _) = DateTime::parse(lexemes.as_slice()).unwrap();
        let res = parsed.to_chrono(now).unwrap();

        assert_eq!(res.hour(), 3);
        assert_eq!(res.minute(), 30);
        assert_eq!(
            res.naive_local()
                .signed_duration_since(res.naive_utc())
                .num_seconds(),
            -4 * 3600
        );
    }

    #[test]
    fn test_datetime_duration_minutes() {
        let lexemes = vec![Lexeme::Num(3), Lexeme::Minute, Lexeme::Before, Lexeme::Now];
        let now = Local
            .with_ymd_and_hms(2021, 4, 30, 7, 15, 17)
            .single()
            .expect("literal date for test case");

        let (date, tokens) = DateTime::parse(lexemes.as_slice()).unwrap();
        let date = date.to_chrono(now);
        let date = date.unwrap();

        assert_eq!(tokens, 4);
        assert_eq!(now - date, chrono::Duration::minutes(3));
    }

    #[test]
    fn test_duration_sub_daily_specific() {
        assert!(!Duration::Specific(2, Unit::Day).is_sub_daily());
        assert!(!Duration::Specific(2, Unit::Week).is_sub_daily());
        assert!(!Duration::Specific(2, Unit::Month).is_sub_daily());
        assert!(!Duration::Specific(2, Unit::Year).is_sub_daily());
        assert!(Duration::Specific(2, Unit::Hour).is_sub_daily());
        assert!(Duration::Specific(2, Unit::Minute).is_sub_daily());
    }

    #[test]
    fn test_duration_sub_daily_article() {
        assert!(!Duration::Article(Unit::Day).is_sub_daily());
        assert!(!Duration::Article(Unit::Week).is_sub_daily());
        assert!(!Duration::Article(Unit::Month).is_sub_daily());
        assert!(!Duration::Article(Unit::Year).is_sub_daily());
        assert!(Duration::Article(Unit::Hour).is_sub_daily());
        assert!(Duration::Article(Unit::Minute).is_sub_daily());
    }

    #[test]
    fn test_duration_sub_daily_concat() {
        assert!(!Duration::Concat(
            Box::new(Duration::Specific(3, Unit::Day)),
            Box::new(Duration::Specific(1, Unit::Year))
        )
        .is_sub_daily());

        assert!(!Duration::Concat(
            Box::new(Duration::Specific(3, Unit::Day)),
            Box::new(Duration::Specific(1, Unit::Week))
        )
        .is_sub_daily());

        assert!(!Duration::Concat(
            Box::new(Duration::Specific(3, Unit::Week)),
            Box::new(Duration::Specific(1, Unit::Year))
        )
        .is_sub_daily());

        assert!(!Duration::Concat(
            Box::new(Duration::Specific(1, Unit::Day)),
            Box::new(Duration::Concat(
                Box::new(Duration::Specific(2, Unit::Week)),
                Box::new(Duration::Specific(3, Unit::Year))
            ))
        )
        .is_sub_daily());

        assert!(Duration::Concat(
            Box::new(Duration::Specific(3, Unit::Hour)),
            Box::new(Duration::Specific(1, Unit::Minute))
        )
        .is_sub_daily());

        assert!(Duration::Concat(
            Box::new(Duration::Specific(3, Unit::Day)),
            Box::new(Duration::Specific(1, Unit::Hour))
        )
        .is_sub_daily());

        assert!(Duration::Concat(
            Box::new(Duration::Specific(1, Unit::Day)),
            Box::new(Duration::Concat(
                Box::new(Duration::Specific(2, Unit::Minute)),
                Box::new(Duration::Specific(3, Unit::Year))
            ))
        )
        .is_sub_daily());
    }
}
