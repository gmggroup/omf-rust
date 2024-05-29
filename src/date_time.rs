//! Utility functions for date and date-time conversion.

use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};

/// Convert a date to the number of days since the epoch.
pub fn date_to_f64(date: NaiveDate) -> f64 {
    date_to_i64(date) as f64
}

/// Convert a date-time the number of seconds since the epoch, including fractional seconds,
/// at a bit less than microsecond precision.
pub fn date_time_to_f64(date_time: DateTime<Utc>) -> f64 {
    const MICRO: i64 = 1_000_000;
    let us = date_time_to_i64(date_time);
    let s = us / MICRO;
    let f = us % MICRO;
    (s as f64) + (f as f64) / (MICRO as f64)
}

/// Convert a date to the number of days since the epoch.
pub fn date_to_i64(date: NaiveDate) -> i64 {
    date.signed_duration_since(NaiveDate::default()).num_days()
}

/// Convert a date-time to the number of microseconds since the epoch.
pub fn date_time_to_i64(date_time: DateTime<Utc>) -> i64 {
    date_time.timestamp_micros()
}

/// Convert a number of days since the epoch back to a date.
pub fn i64_to_date(value: i64) -> NaiveDate {
    Duration::try_days(value)
        .and_then(|d| NaiveDate::default().checked_add_signed(d))
        .unwrap_or(if value < 0 {
            NaiveDate::MIN
        } else {
            NaiveDate::MAX
        })
}

/// Convert a number of microseconds since the epoch back to a date.
pub fn i64_to_date_time(value: i64) -> DateTime<Utc> {
    DateTime::<Utc>::default()
        .checked_add_signed(Duration::microseconds(value))
        .unwrap_or(if value < 0 {
            DateTime::<Utc>::MIN_UTC
        } else {
            DateTime::<Utc>::MAX_UTC
        })
}

/// Convert a number of milliseconds since the epoch back to a date.
pub fn i64_milli_to_date_time(value: i64) -> DateTime<Utc> {
    Duration::try_milliseconds(value)
        .and_then(|d| DateTime::<Utc>::default().checked_add_signed(d))
        .unwrap_or(if value < 0 {
            DateTime::<Utc>::MIN_UTC
        } else {
            DateTime::<Utc>::MAX_UTC
        })
}

/// Convert a number of nanoseconds since the epoch back to a date.
pub fn i64_nano_to_date_time(value: i64) -> DateTime<Utc> {
    DateTime::<Utc>::default()
        .checked_add_signed(Duration::nanoseconds(value))
        .unwrap_or(if value < 0 {
            DateTime::<Utc>::MIN_UTC
        } else {
            DateTime::<Utc>::MAX_UTC
        })
}

/// Returns the current date and time in UTC.
pub fn utc_now() -> DateTime<Utc> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("valid system time");
    let naive = DateTime::from_timestamp(now.as_secs() as i64, now.subsec_nanos())
        .expect("valid timestamp")
        .naive_utc();
    Utc.from_utc_datetime(&naive)
}
