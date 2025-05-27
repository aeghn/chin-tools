use postgres_types::ToSql;

use super::SqlValue;

impl<'a> Into<&'a (dyn ToSql + Sync + Send)> for &'a SqlValue<'a> {
    fn into(self) -> &'a (dyn ToSql + Sync + Send) {
        match self {
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
                None => &None::<String>,
            },
            SqlValue::F64(v) => v,
            SqlValue::Blob(cow) => cow,
        }
    }
}
