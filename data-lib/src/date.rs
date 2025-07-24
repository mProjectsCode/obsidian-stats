use chrono::Datelike;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Date {
    pub year: u32,
    pub month: u32,
    pub day: u32,
}

impl Date {
    pub fn new(year: u32, month: u32, day: u32) -> Self {
        Date { year, month, day }
    }

    pub fn from_string(date_str: &str) -> Option<Self> {
        let parts: Vec<&str> = date_str.split('-').collect();
        if parts.len() == 3
            && let (Ok(year), Ok(month), Ok(day)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            )
        {
            return Some(Date::new(year, month, day));
        }

        None
    }

    pub fn now() -> Self {
        let now = chrono::Utc::now();
        Date::new(now.year() as u32, now.month(), now.day())
    }

    pub fn to_fancy_string(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    pub fn in_range(&self, start: &Date, end: &Date) -> bool {
        self >= start && self <= end
    }

    pub fn get_month_length(&self) -> u32 {
        match self.month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if self.is_leap_year() {
                    29
                } else {
                    28
                }
            }
            _ => panic!("Invalid month"),
        }
    }

    pub fn is_leap_year(&self) -> bool {
        (self.year % 4 == 0 && self.year % 100 != 0) || (self.year % 400 == 0)
    }

    pub fn days_since_epoch(&self) -> u32 {
        let mut days = 0;
        for y in 1970..self.year {
            days += if Date::new(y, 1, 1).is_leap_year() {
                366
            } else {
                365
            };
        }
        for m in 1..self.month {
            days += Date::new(self.year, m, 1).get_month_length();
        }
        days + self.day - 1 // Subtract one because we start counting from day 1
    }

    pub fn week_day(&self) -> u32 {
        let days = self.days_since_epoch();
        ((days + 4) % 7) + 1 // 1970-01-01 was a Thursday, so we adjust to make it 1-based
    }

    pub fn advance_days(&mut self, days: u32) {
        let mut day = self.day + days;
        let mut month = self.month;
        let mut year = self.year;

        while day > self.get_month_length() {
            day -= self.get_month_length();
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
        }

        self.day = day;
        self.month = month;
        self.year = year;
    }

    pub fn reverse_days(&mut self, days: u32) {
        let mut day = self.day;
        let mut month = self.month;
        let mut year = self.year;

        for _ in 0..days {
            if day == 1 {
                month -= 1;
                if month == 0 {
                    month = 12;
                    year -= 1;
                }
                day = Date::new(year, month, 1).get_month_length();
            } else {
                day -= 1;
            }
        }

        self.day = day;
        self.month = month;
        self.year = year;
    }

    pub fn advance_day(&mut self) {
        self.advance_days(1);
    }

    pub fn advance_week(&mut self) {
        self.advance_days(7);
    }

    pub fn advance_month(&mut self) {
        self.day = 1; // Reset to the first day of the month
        self.month += 1;
        if self.month > 12 {
            self.month = 1;
            self.year += 1;
        }
    }

    pub fn advance_to_weekday(&mut self, week_day: u32) {
        let current_week_day = self.week_day();
        if week_day < current_week_day {
            // Move to next week
            self.advance_days(7 - (current_week_day - week_day));
        } else if week_day > current_week_day {
            // Move to same week
            self.advance_days(week_day - current_week_day);
        }
    }

    pub fn diff_in_days(&self, other: &Date) -> i32 {
        let self_days = self.days_since_epoch();
        let other_days = other.days_since_epoch();
        self_days as i32 - other_days as i32
    }

    pub fn iterate_daily_to<'a>(&'a self, end: &'a Date) -> impl Iterator<Item = Date> + 'a {
        let mut current = self.clone();
        std::iter::from_fn(move || {
            if &current <= end {
                let next = current.clone();
                current.advance_day();
                Some(next)
            } else {
                None
            }
        })
    }

    pub fn iterate_daily_backwards<'a>(
        &'a self,
        start: &'a Date,
    ) -> impl Iterator<Item = Date> + 'a {
        let mut current = self.clone();
        std::iter::from_fn(move || {
            if &current >= start {
                let next = current.clone();
                current.reverse_days(1);
                Some(next)
            } else {
                None
            }
        })
    }

    pub fn iterate_weekly_to<'a>(&'a self, end: &'a Date) -> impl Iterator<Item = Date> + 'a {
        let mut current = self.clone();
        std::iter::from_fn(move || {
            if &current <= end {
                let next = current.clone();
                current.advance_week();
                Some(next)
            } else {
                None
            }
        })
    }

    pub fn iterate_monthly_to<'a>(&'a self, end: &'a Date) -> impl Iterator<Item = Date> + 'a {
        let mut current = self.clone();
        std::iter::from_fn(move || {
            if &current <= end {
                let next = current.clone();
                current.advance_month();
                Some(next)
            } else {
                None
            }
        })
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl std::str::FromStr for Date {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Date::from_string(s).ok_or("Invalid date format")
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Date {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.year, self.month, self.day).cmp(&(other.year, other.month, other.day))
    }
}

#[test]
fn date_compare() {
    let date1 = Date::new(2023, 10, 1);
    let date2 = Date::new(2023, 10, 2);
    let date3 = Date::new(2024, 1, 1);

    assert!(date1 < date2);
    assert!(date2 > date1);
    assert!(date1 < date3);
    assert!(date3 > date2);
}

#[test]
fn date_to_string() {
    let date1 = Date::new(2023, 10, 1);
    let date2 = Date::new(2023, 10, 2);
    let date3 = Date::new(2024, 1, 1);

    assert_eq!(date1.to_fancy_string(), "2023-10-01");
    assert_eq!(date2.to_fancy_string(), "2023-10-02");
    assert_eq!(date3.to_fancy_string(), "2024-01-01");
}

#[test]
fn date_advance() {
    let mut date4 = Date::new(2023, 10, 31);
    date4.advance_day();
    assert_eq!(date4.to_fancy_string(), "2023-11-01");

    let mut date5 = Date::new(2023, 12, 31);
    date5.advance_month();
    assert_eq!(date5.to_fancy_string(), "2024-01-01");
}
