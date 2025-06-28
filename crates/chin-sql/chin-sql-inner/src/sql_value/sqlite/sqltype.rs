use std::ops::{Deref, DerefMut};

use chrono::{
    DateTime, Datelike, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc,
};

use crate::ChinSqlError;

#[derive(Clone, Debug)]
pub struct Timestamptz(pub(super) DateTime<FixedOffset>);

impl From<DateTime<FixedOffset>> for Timestamptz {
    fn from(value: DateTime<FixedOffset>) -> Self {
        Self(value)
    }
}

impl From<DateTime<Utc>> for Timestamptz {
    fn from(value: DateTime<Utc>) -> Self {
        Self(value.fixed_offset())
    }
}

impl Deref for Timestamptz {
    type Target = DateTime<FixedOffset>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Timestamptz {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Timestamptz> for DateTime<FixedOffset> {
    fn from(value: Timestamptz) -> Self {
        value.0
    }
}

// 9223372036854775807
// yyyydddsssssSSSzone
// {yyyy,4}{oridinal,3}{sincemidnightseconds,5,0~86400}{millsesonds,3}{timezone,4,5000+-1400}
impl TryFrom<i64> for Timestamptz {
    type Error = ChinSqlError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let raw_tz = (value % 10000 - 5000) as i32;
        let tz = if raw_tz > 0 {
            FixedOffset::east_opt(raw_tz * 60)
        } else {
            FixedOffset::west_opt(raw_tz * 60)
        };
        let tz = tz.ok_or(ChinSqlError::TransformError(format!(
            "unable to extract tz from {raw_tz} -- {value}"
        )))?;

        let raw_time = value / 10000;
        let mills = (raw_time % 1000) as i32;
        let seconds = (raw_time / 1000) % 100000;

        let naive_time =
            NaiveTime::from_num_seconds_from_midnight_opt(seconds as u32, mills as u32 * 1000000)
                .ok_or(ChinSqlError::TransformError(
                "unable to build naive time".to_owned(),
            ))?;

        let yo = raw_time / 100000000;
        let y = yo / 1000;
        let o = yo % 1000;

        let navie_date = NaiveDate::from_yo_opt(
            y.try_into().map_err(|err: std::num::TryFromIntError| {
                ChinSqlError::TransformError(err.to_string())
            })?,
            o.try_into().map_err(|err: std::num::TryFromIntError| {
                ChinSqlError::TransformError(err.to_string())
            })?,
        )
        .ok_or(ChinSqlError::TransformError(
            "unable to build naive time".to_owned(),
        ))?;

        Ok(Self(tz.from_utc_datetime(&NaiveDateTime::new(
            navie_date, naive_time,
        ))))
    }
}

impl From<Timestamptz> for i64 {
    fn from(val: Timestamptz) -> Self {
        let value = val.0;

        let tz = value.offset();

        let value = value.to_utc();
        let date = value.date_naive();
        let time = value.time();

        let tz = tz.local_minus_utc() as i64 / 60 + 5000;

        let mills = time.nanosecond() as i64 / 10_i64.pow(6);

        let time = time
            .signed_duration_since(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .num_seconds();

        let date: i64 = (if date.year_ce().0 { 1 } else { -1 }) as i64
            * (date.ordinal0() + 1 + date.year_ce().1 * 1000) as i64;

        tz + mills * 10000 + time * 10000000 + date * 1000000000000
    }
}

#[cfg(test)]
pub mod tests {
    use chrono::{FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};

    use crate::ChinSqlError;

    use super::Timestamptz;

    #[test]
    fn convert() -> Result<(), ChinSqlError> {
        let tz = FixedOffset::east_opt(9 * 60 * 60).unwrap();
        let naive_time = NaiveTime::from_num_seconds_from_midnight_opt(
            12 * 60 * 60 + 12 * 60 + 12,
            999 * 1000000,
        )
        .unwrap();
        let navie_date = NaiveDate::from_ymd_opt(2020, 12, 30).unwrap();
        let datetime = tz.from_utc_datetime(&NaiveDateTime::new(navie_date, naive_time));

        let original = 2020365439329995540;

        let tz: Timestamptz = original.try_into()?;
        let tzn: i64 = tz.clone().into();
        println!("{} -- {}", tz.0, tzn);
        assert!(tzn == original);
        assert!(tz.0 == datetime);

        Ok(())
    }
}
