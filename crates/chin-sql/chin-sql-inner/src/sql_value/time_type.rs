use std::{
    fmt::{Display, Formatter},
    sync::{
        Arc, LazyLock,
        atomic::{AtomicI64, Ordering},
    },
};

use chrono::{DateTime, FixedOffset, Local, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serialize};

pub fn current_timestamptz() -> DateTime<FixedOffset> {
    Utc::now().with_timezone(&TimeZone::from_offset(Local::now().offset()))
}

pub const TID_NEVER: i64 = -404;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TID(i64);

impl Display for TID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.to_string().as_str())
    }
}

static COUNTER: LazyLock<Arc<AtomicI64>> = LazyLock::new(|| Arc::new(AtomicI64::new(0)));

impl Default for TID {
    fn default() -> Self {
        loop {
            let time = Utc::now().timestamp_millis() * 1000;
            let current = COUNTER.load(std::sync::atomic::Ordering::SeqCst);
            let new = if time > current {
                time + rand::random_range(1..1000)
            } else {
                current + 1
            };

            if COUNTER
                .compare_exchange(
                    current,
                    new,
                    std::sync::atomic::Ordering::SeqCst,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                return Self(new);
            }
        }
    }
}

impl Serialize for TID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i64(self.0)
    }
}

impl<'de> Deserialize<'de> for TID {
    fn deserialize<D>(deserializer: D) -> Result<TID, D::Error>
    where
        D: Deserializer<'de>,
    {
        let o: i64 = i64::deserialize(deserializer)?;
        Ok(Self(o))
    }
}

impl From<i64> for TID {
    fn from(value: i64) -> Self {
        Self::check(value);
        Self(value)
    }
}

impl From<DateTime<Utc>> for TID {
    fn from(value: DateTime<Utc>) -> Self {
        let num = value.timestamp_micros();
        Self::check(num);
        Self(num)
    }
}

impl TID {
    #[inline]
    pub fn as_utc(&self) -> DateTime<Utc> {
        (*self).into()
    }

    #[inline]
    pub fn as_num(&self) -> i64 {
        self.0
    }

    #[inline]
    pub fn is_never(&self) -> bool {
        self.0 == TID_NEVER
    }

    #[inline]
    pub fn never() -> Self {
        Self(TID_NEVER)
    }
}

impl From<DateTime<FixedOffset>> for TID {
    fn from(value: DateTime<FixedOffset>) -> Self {
        let num = value.timestamp_micros();
        Self::check(num);
        Self(num)
    }
}

impl From<TID> for i64 {
    fn from(value: TID) -> Self {
        value.0
    }
}

impl From<TID> for DateTime<Utc> {
    fn from(value: TID) -> Self {
        Utc.timestamp_opt(value.0 / 1_000, (value.0 % 1_000_000 * 1000) as u32)
            .unwrap()
    }
}

impl TID {
    fn check(num: i64) {
        // Javascript max safe number
        assert!(num < 9_007_199_254_740_991 || num == TID_NEVER);
    }
}

pub mod from_pg {
    use postgres_types::{FromSql, accepts};

    use crate::time_type::TID;

    impl<'a> FromSql<'a> for TID {
        fn from_sql(
            ty: &postgres_types::Type,
            raw: &'a [u8],
        ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            i64::from_sql(ty, raw).map(Self)
        }

        accepts! {INT2, INT4, INT8}
    }
}

#[cfg(test)]
mod tests {
    use chrono::{FixedOffset, Local, TimeZone, Utc};

    #[test]
    fn test() {
        let fixed = Utc::now();
        println!("{:#?} -> {}", fixed, fixed.timestamp_millis());
        let fixed: chrono::DateTime<FixedOffset> =
            Utc::now().with_timezone(&TimeZone::from_offset(Local::now().offset()));
        println!("{:#?} -> {}", fixed, fixed.timestamp_millis());
    }

    use crate::sql_value::TID;
    #[test]
    fn test_generate() {
        for _ in 1..10000 {
            let c = TID::default();
            if c.as_num() % 500 == 0 {
                print!("{c}, ");
            }
        }
    }
}
