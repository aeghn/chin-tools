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

impl<'a> Into<SqlValue<'a>> for SStr {
    fn into(self) -> SqlValue<'a> {
        SqlValue::SharedStr(SharedStr(self))
    }
}

impl<'a> Into<SqlValue<'a>> for bool {
    fn into(self) -> SqlValue<'a> {
        SqlValue::Bool(self)
    }
}

impl<'a> Into<SqlValue<'a>> for DateTime<FixedOffset> {
    fn into(self) -> SqlValue<'a> {
        SqlValue::FixedOffset(DateFixedOffset(self))
    }
}

impl<'a> Into<SqlValue<'a>> for &'a DateTime<FixedOffset> {
    fn into(self) -> SqlValue<'a> {
        SqlValue::FixedOffset(DateFixedOffset(*self))
    }
}

impl<'a> Into<SqlValue<'a>> for &'a DateTime<Utc> {
    fn into(self) -> SqlValue<'a> {
        SqlValue::Utc(DateUtc(*self))
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
