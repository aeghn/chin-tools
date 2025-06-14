use core::prelude::v1;
use std::{any::Any, sync::atomic::AtomicI16};

use chrono::{DateTime, FixedOffset, Utc};
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
            SqlValue::F64(v) => v,
            SqlValue::Blob(cow) => cow,
            SqlValue::Null(rust_field_type) => match rust_field_type {
                super::RustFieldType::Bool => &None::<bool>,
                super::RustFieldType::I8 => &None::<i8>,
                super::RustFieldType::I16 => &None::<i16>,
                super::RustFieldType::I32 => &None::<i32>,
                super::RustFieldType::I64 => &None::<i64>,
                super::RustFieldType::F64 => &None::<f64>,
                super::RustFieldType::Text => &None::<String>,
                super::RustFieldType::Blob => &None::<Vec<u8>>,
                super::RustFieldType::Timestamptz => &None::<DateTime<FixedOffset>>,
                super::RustFieldType::Timestamp => &None::<DateTime<Utc>>,
                super::RustFieldType::Any => unreachable!(),
            },
        }
    }
}
