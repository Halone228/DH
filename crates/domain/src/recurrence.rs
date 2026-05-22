use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

impl From<Weekday> for chrono::Weekday {
    fn from(w: Weekday) -> Self {
        match w {
            Weekday::Mon => chrono::Weekday::Mon,
            Weekday::Tue => chrono::Weekday::Tue,
            Weekday::Wed => chrono::Weekday::Wed,
            Weekday::Thu => chrono::Weekday::Thu,
            Weekday::Fri => chrono::Weekday::Fri,
            Weekday::Sat => chrono::Weekday::Sat,
            Weekday::Sun => chrono::Weekday::Sun,
        }
    }
}

/// Recurrence as a closed enum. Picking an enum (rather than RFC-5545 RRULE)
/// keeps the storage cheap and the UI simple. If users ever ask for arbitrary
/// rules we can add a `Rrule { spec: String }` variant alongside — open for
/// extension, no migration required for existing kinds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Recurrence {
    Once {
        at: DateTime<Utc>,
    },
    Daily {
        time: NaiveTime,
    },
    Weekly {
        weekdays: Vec<Weekday>,
        time: NaiveTime,
    },
    Monthly {
        day_of_month: u8,
        time: NaiveTime,
    },
}

impl Recurrence {
    /// First fire-time strictly after `after`, expressed in UTC.
    /// Returns `None` if the recurrence has no future occurrence
    /// (e.g. a `Once` whose datetime has already passed).
    pub fn next_after(&self, after: DateTime<Utc>, tz: Tz) -> Option<DateTime<Utc>> {
        match self {
            Recurrence::Once { at } => (*at > after).then_some(*at),
            Recurrence::Daily { time } => Some(next_daily(after, tz, *time)),
            Recurrence::Weekly { weekdays, time } => next_weekly(after, tz, weekdays, *time),
            Recurrence::Monthly {
                day_of_month,
                time,
            } => next_monthly(after, tz, *day_of_month, *time),
        }
    }
}

fn next_daily(after: DateTime<Utc>, tz: Tz, time: NaiveTime) -> DateTime<Utc> {
    let mut date = after.with_timezone(&tz).date_naive();
    loop {
        if let Some(candidate) = combine(date, time, tz) {
            if candidate > after {
                return candidate;
            }
        }
        date = date.succ_opt().expect("date overflow");
    }
}

fn next_weekly(
    after: DateTime<Utc>,
    tz: Tz,
    weekdays: &[Weekday],
    time: NaiveTime,
) -> Option<DateTime<Utc>> {
    if weekdays.is_empty() {
        return None;
    }
    let mut date = after.with_timezone(&tz).date_naive();
    for _ in 0..14 {
        if weekdays
            .iter()
            .any(|w| chrono::Weekday::from(*w) == date.weekday())
        {
            if let Some(candidate) = combine(date, time, tz) {
                if candidate > after {
                    return Some(candidate);
                }
            }
        }
        date = date.succ_opt()?;
    }
    None
}

fn next_monthly(
    after: DateTime<Utc>,
    tz: Tz,
    day_of_month: u8,
    time: NaiveTime,
) -> Option<DateTime<Utc>> {
    let local = after.with_timezone(&tz);
    let mut year = local.year();
    let mut month = local.month();

    for _ in 0..24 {
        let day = clamp_day_to_month(year, month, day_of_month);
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            if let Some(candidate) = combine(date, time, tz) {
                if candidate > after {
                    return Some(candidate);
                }
            }
        }
        month += 1;
        if month > 12 {
            month = 1;
            year += 1;
        }
    }
    None
}

fn combine(date: NaiveDate, time: NaiveTime, tz: Tz) -> Option<DateTime<Utc>> {
    let naive = date.and_time(time);
    // DST transitions can map a wall-clock time to zero or two instants.
    // Pick the earliest valid one; if the time is invalid (skipped during
    // DST forward), fall forward by one minute and retry.
    match tz.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => Some(dt.with_timezone(&Utc)),
        chrono::LocalResult::Ambiguous(a, _) => Some(a.with_timezone(&Utc)),
        chrono::LocalResult::None => {
            let bumped = naive + Duration::minutes(1);
            tz.from_local_datetime(&bumped)
                .single()
                .map(|dt| dt.with_timezone(&Utc))
        }
    }
}

fn clamp_day_to_month(year: i32, month: u32, requested: u8) -> u32 {
    let last = last_day_of_month(year, month);
    (requested as u32).min(last).max(1)
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .and_then(|d| d.pred_opt())
        .map(|d| d.day())
        .unwrap_or(28)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Timelike};

    fn moscow() -> Tz {
        chrono_tz::Europe::Moscow
    }

    #[test]
    fn once_returns_none_when_in_the_past() {
        let r = Recurrence::Once {
            at: Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
        };
        assert!(r.next_after(Utc::now(), moscow()).is_none());
    }

    #[test]
    fn daily_advances_to_next_day_when_today_passed() {
        let now = moscow()
            .with_ymd_and_hms(2026, 5, 3, 10, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        let r = Recurrence::Daily {
            time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        };
        let next = r.next_after(now, moscow()).unwrap().with_timezone(&moscow());
        assert_eq!(next.day(), 4);
        assert_eq!(next.hour(), 9);
    }

    #[test]
    fn monthly_clamps_day_31_in_february() {
        let now = moscow()
            .with_ymd_and_hms(2026, 1, 31, 12, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        let r = Recurrence::Monthly {
            day_of_month: 31,
            time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        };
        let next = r.next_after(now, moscow()).unwrap().with_timezone(&moscow());
        assert_eq!(next.month(), 2);
        assert_eq!(next.day(), 28);
    }
}
