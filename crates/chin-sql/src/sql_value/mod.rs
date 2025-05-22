#[cfg(feature = "sqlite")]
mod sqlite;

#[cfg(feature = "postgres")]
mod postgres;

use std::{borrow::Cow, ops::Deref};

use chrono::{DateTime, FixedOffset, Utc};

type SStr = chin_tools_types::SharedStr;

#[derive(Clone, Debug)]
pub struct SharedStr(SStr);

impl Deref for SharedStr {
    type Target = SStr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct DateFixedOffset(DateTime<FixedOffset>);

impl From<DateTime<FixedOffset>> for DateFixedOffset {
    fn from(value: DateTime<FixedOffset>) -> Self {
        Self(value)
    }
}

impl Deref for DateFixedOffset {
    type Target = DateTime<FixedOffset>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct DateUtc(DateTime<Utc>);

#[derive(Clone, Debug)]
pub enum SqlValue<'a> {
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F64(f64),
    Str(Cow<'a, str>),
    SharedStr(SharedStr),
    FixedOffset(DateFixedOffset),
    Utc(DateUtc),
    Blob(Cow<'a, [u8]>),
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
            SqlValue::FixedOffset(v) => SqlValue::FixedOffset(v),
            SqlValue::Utc(v) => SqlValue::Utc(v),
            SqlValue::Bool(v) => SqlValue::Bool(v),
            SqlValue::SharedStr(v) => SqlValue::SharedStr(v),
            SqlValue::F64(v) => SqlValue::F64(v),
            SqlValue::Blob(cow) => SqlValue::Blob(Cow::Owned(cow.to_vec())),
        }
    }
}

impl<'a> From<i8> for SqlValue<'a> {
    fn from(val: i8) -> Self {
        SqlValue::I8(val)
    }
}

impl<'a> From<i16> for SqlValue<'a> {
    fn from(val: i16) -> Self {
        SqlValue::I16(val)
    }
}

impl<'a> From<i32> for SqlValue<'a> {
    fn from(val: i32) -> Self {
        SqlValue::I32(val)
    }
}

impl<'a> From<i64> for SqlValue<'a> {
    fn from(val: i64) -> Self {
        SqlValue::I64(val)
    }
}

impl<'a> From<&'a String> for SqlValue<'a> {
    fn from(val: &'a String) -> Self {
        SqlValue::Str(Cow::Borrowed(val))
    }
}

impl<'a> From<&'a str> for SqlValue<'a> {
    fn from(val: &'a str) -> Self {
        SqlValue::Str(Cow::Borrowed(val))
    }
}

impl<'a> From<String> for SqlValue<'a> {
    fn from(val: String) -> Self {
        SqlValue::Str(Cow::Owned(val))
    }
}

impl<'a> From<Cow<'a, str>> for SqlValue<'a> {
    fn from(val: Cow<'a, str>) -> Self {
        SqlValue::Str(val)
    }
}

impl<'a> From<SStr> for SqlValue<'a> {
    fn from(val: SStr) -> Self {
        SqlValue::SharedStr(SharedStr(val))
    }
}

impl<'a> From<bool> for SqlValue<'a> {
    fn from(val: bool) -> Self {
        SqlValue::Bool(val)
    }
}

impl<'a> From<DateTime<FixedOffset>> for SqlValue<'a> {
    fn from(val: DateTime<FixedOffset>) -> Self {
        SqlValue::FixedOffset(DateFixedOffset(val))
    }
}

impl<'a> From<&'a DateTime<FixedOffset>> for SqlValue<'a> {
    fn from(val: &'a DateTime<FixedOffset>) -> Self {
        SqlValue::FixedOffset(DateFixedOffset(*val))
    }
}

impl<'a> From<&'a DateTime<Utc>> for SqlValue<'a> {
    fn from(val: &'a DateTime<Utc>) -> Self {
        SqlValue::Utc(DateUtc(*val))
    }
}

impl<'a, T: Into<SqlValue<'a>>> From<Option<T>> for SqlValue<'a> {
    fn from(val: Option<T>) -> Self {
        SqlValue::Opt(val.map(|e| {
            let sv: SqlValue<'a> = e.into();
            sv.into()
        }))
    }
}
