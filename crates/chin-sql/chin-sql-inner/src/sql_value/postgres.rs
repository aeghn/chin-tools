use chrono::{DateTime, FixedOffset, Utc};
use postgres_types::ToSql;

use crate::{LogicFieldType, SqlValue};

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
                LogicFieldType::Bool => &None::<bool>,
                LogicFieldType::I8 => &None::<i8>,
                LogicFieldType::I16 => &None::<i16>,
                LogicFieldType::I32 => &None::<i32>,
                LogicFieldType::I64 => &None::<i64>,
                LogicFieldType::F64 => &None::<f64>,
                LogicFieldType::Text => &None::<String>,
                LogicFieldType::Blob => &None::<Vec<u8>>,
                LogicFieldType::Timestamptz => &None::<DateTime<FixedOffset>>,
                LogicFieldType::Timestamp => &None::<DateTime<Utc>>,
                LogicFieldType::Varchar(_) => &None::<String>,
            },
            SqlValue::NullUnknown => unreachable!(),
        }
    }
}
