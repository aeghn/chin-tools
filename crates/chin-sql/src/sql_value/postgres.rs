use core::prelude::v1;
use std::any::Any;

use chrono::{DateTime, FixedOffset};
use postgres_types::ToSql;

use super::SqlValue;

impl<'a> From<&'a SqlValue<'a>> for &'a (dyn ToSql + Sync + Send) {
    fn from(val: &'a SqlValue<'a>) -> Self {
        match val {
            SqlValue::I8(v) => v,
            SqlValue::I16(v) => v,
            SqlValue::I32(v) => v,
            SqlValue::I64(v) => v,
            SqlValue::Str(v) => v,
            SqlValue::FixedOffset(v) => v,
            SqlValue::Utc(v) => v,
            SqlValue::Bool(v) => v,
            SqlValue::Opt(v) => match v {
                Some(v) => v.as_ref().into(),
                None => &None::<DateTime<FixedOffset>>,
            },
            SqlValue::F64(v) => v,
            SqlValue::Blob(cow) => cow,
        }
    }
}
