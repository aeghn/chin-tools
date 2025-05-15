use std::error::Error;

use bytes::BytesMut;
use chrono::{DateTime, FixedOffset, Utc};
use postgres_types::{FromSql, ToSql, Type, accepts, to_sql_checked};

use super::{DateFixedOffset, DateUtc, SharedStr, SqlValue};

impl ToSql for DateFixedOffset {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        self.0.to_sql(ty, out)
    }

    accepts!(TIMESTAMPTZ);

    to_sql_checked!();
}

impl ToSql for SharedStr {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        self.0.as_str().to_sql(ty, out)
    }

    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        <&str as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}

impl ToSql for DateUtc {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        self.0.to_sql(ty, out)
    }

    accepts!(TIMESTAMPTZ);
    to_sql_checked!();
}

impl<'a> FromSql<'a> for DateFixedOffset {
    fn from_sql(type_: &Type, raw: &[u8]) -> Result<DateFixedOffset, Box<dyn Error + Sync + Send>> {
        let utc = DateTime::<Utc>::from_sql(type_, raw)?;
        Ok(DateFixedOffset(
            utc.with_timezone(&FixedOffset::east_opt(0).unwrap()),
        ))
    }

    accepts!(TIMESTAMPTZ);
}

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
            SqlValue::SharedStr(v) => v,
            SqlValue::F64(v) => v,
            SqlValue::Blob(cow) => cow,
        }
    }
}