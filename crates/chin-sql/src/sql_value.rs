use std::borrow::Cow;

use chrono::{DateTime, FixedOffset, Utc};

use chin_tools_base::SharedStr;

#[derive(Clone, Debug)]
pub enum SqlValue<'a> {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Str(Cow<'a, str>),
    SharedStr(SharedStr),
    Date(Cow<'a, DateTime<FixedOffset>>),
    DateUtc(Cow<'a, DateTime<Utc>>),
    Bool(bool),
    Opt(Option<Box<SqlValue<'a>>>),
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
        SqlValue::SharedStr(self)
    }
}

impl<'a> Into<SqlValue<'a>> for bool {
    fn into(self) -> SqlValue<'a> {
        SqlValue::Bool(self)
    }
}

impl<'a> Into<SqlValue<'a>> for DateTime<FixedOffset> {
    fn into(self) -> SqlValue<'a> {
        SqlValue::Date(Cow::Owned(self))
    }
}

impl<'a> Into<SqlValue<'a>> for &'a DateTime<FixedOffset> {
    fn into(self) -> SqlValue<'a> {
        SqlValue::Date(Cow::Borrowed(self))
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
                SqlValue::Date(v) => v.as_ref(),
                SqlValue::DateUtc(v) => v.as_ref(),
                SqlValue::Bool(v) => v,
                SqlValue::Opt(v) => match v {
                    Some(v) => v.as_ref().into(),
                    None => &None::<String>,
                },
                SqlValue::SharedStr(shared_str) => shared_str,
            }
        }
    }
}
