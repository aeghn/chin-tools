#[cfg(feature = "sqlite")]
mod sqlite;

use std::{borrow::Cow, ops::Deref};

use chrono::{DateTime, FixedOffset, Utc};

use chin_tools_types::SharedStr;

#[derive(Clone, Debug)]
pub struct ShareStr(SharedStr);

impl Deref for ShareStr {
    type Target = SharedStr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct DateFixed(DateTime<FixedOffset>);

impl From<DateTime<FixedOffset>> for DateFixed {
    fn from(value: DateTime<FixedOffset>) -> Self {
        Self(value)
    }
}

impl Deref for DateFixed {
    type Target = DateTime<FixedOffset>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct DateUtc(DateTime<Utc>);

#[derive(Clone, Debug)]
pub enum SqlValue<'a> {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Str(Cow<'a, str>),
    SharedStr(ShareStr),
    Date(DateFixed),
    DateUtc(DateUtc),
    Bool(bool),
    Opt(Option<Box<SqlValue<'a>>>),
}

pub struct SqlValueOwned(SqlValue<'static>);

impl<'a> From<SqlValue<'a>> for SqlValueOwned {
    fn from(value: SqlValue<'a>) -> Self {
        Self(value.live_static())
    }
}

impl<'a> SqlValue<'a> {
    pub fn live_static(self) -> SqlValue<'static> {
        match self {
            SqlValue::Str(v) => SqlValue::Str(Cow::Owned(v.to_string())),
            SqlValue::Opt(v) => match v {
                Some(v) => SqlValue::Opt(Some(Box::new(v.live_static()))),
                None => SqlValue::Opt(None),
            },
            SqlValue::I8(v) => SqlValue::I8(v),
            SqlValue::I16(v) => SqlValue::I16(v),
            SqlValue::I32(v) => SqlValue::I32(v),
            SqlValue::I64(v) => SqlValue::I64(v),
            SqlValue::Date(v) => SqlValue::Date(v),
            SqlValue::DateUtc(v) => SqlValue::DateUtc(v),
            SqlValue::Bool(v) => SqlValue::Bool(v),
            SqlValue::SharedStr(v) => SqlValue::SharedStr(v),
        }
    }
}

impl<'a> Into<SqlValue<'a>> for i8 {
    fn into(self) -> SqlValue<'a> {
        SqlValue::I8(self)
    }
}

impl<'a> Into<SqlValue<'a>> for i16 {
    fn into(self) -> SqlValue<'a> {
        SqlValue::I16(self)
    }
}

impl<'a> Into<SqlValue<'a>> for i32 {
    fn into(self) -> SqlValue<'a> {
        SqlValue::I32(self)
    }
}

impl<'a> Into<SqlValue<'a>> for i64 {
    fn into(self) -> SqlValue<'a> {
        SqlValue::I64(self)
    }
}

impl<'a> Into<SqlValue<'a>> for &'a String {
    fn into(self) -> SqlValue<'a> {
        SqlValue::Str(Cow::Borrowed(self))
    }
}

impl<'a> Into<SqlValue<'a>> for &'a str {
    fn into(self) -> SqlValue<'a> {
        SqlValue::Str(Cow::Borrowed(self))
    }
}

impl<'a> Into<SqlValue<'a>> for String {
    fn into(self) -> SqlValue<'a> {
        SqlValue::Str(Cow::Owned(self))
    }
}

impl<'a> Into<SqlValue<'a>> for Cow<'a, str> {
    fn into(self) -> SqlValue<'a> {
        SqlValue::Str(self)
    }
}

impl<'a> Into<SqlValue<'a>> for SharedStr {
    fn into(self) -> SqlValue<'a> {
        SqlValue::SharedStr(ShareStr(self))
    }
}

impl<'a> Into<SqlValue<'a>> for bool {
    fn into(self) -> SqlValue<'a> {
        SqlValue::Bool(self)
    }
}

impl<'a> Into<SqlValue<'a>> for DateTime<FixedOffset> {
    fn into(self) -> SqlValue<'a> {
        SqlValue::Date(DateFixed(self))
    }
}

impl<'a> Into<SqlValue<'a>> for &'a DateTime<FixedOffset> {
    fn into(self) -> SqlValue<'a> {
        SqlValue::Date(DateFixed(*self))
    }
}

impl<'a> Into<SqlValue<'a>> for &'a DateTime<Utc> {
    fn into(self) -> SqlValue<'a> {
        SqlValue::DateUtc(DateUtc(*self))
    }
}

impl<'a, T: Into<SqlValue<'a>>> Into<SqlValue<'a>> for Option<T> {
    fn into(self) -> SqlValue<'a> {
        SqlValue::Opt(self.map(|e| {
            let sv: SqlValue<'a> = e.into();
            sv.into()
        }))
    }
}

#[cfg(feature = "postgres")]
mod postgres {
    use std::error::Error;

    use bytes::BytesMut;
    use chrono::{DateTime, FixedOffset, Utc};
    use postgres_types::{FromSql, ToSql, Type, accepts, to_sql_checked};

    use super::{DateFixed, DateUtc, ShareStr, SqlValue};

    impl ToSql for DateFixed {
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

    impl ToSql for ShareStr {
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

    impl<'a> FromSql<'a> for DateFixed {
        fn from_sql(type_: &Type, raw: &[u8]) -> Result<DateFixed, Box<dyn Error + Sync + Send>> {
            let utc = DateTime::<Utc>::from_sql(type_, raw)?;
            Ok(DateFixed(
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
                SqlValue::Date(v) => v,
                SqlValue::DateUtc(v) => v,
                SqlValue::Bool(v) => v,
                SqlValue::Opt(v) => match v {
                    Some(v) => v.as_ref().into(),
                    None => &None::<String>,
                },
                SqlValue::SharedStr(v) => v,
            }
        }
    }
}
