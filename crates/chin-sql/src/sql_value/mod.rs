#[cfg(feature = "sqlite")]
mod sqlite;

#[cfg(feature = "postgres")]
mod postgres;

use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
    ops::Deref,
    sync::Arc,
};

use chrono::{DateTime, FixedOffset, Utc};
use rusqlite::types::Value;
use sqlite::sqltype::Timestamptz;

use crate::ChinSqlError;

#[derive(Clone, Debug)]
pub enum SqlValue<'a> {
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F64(f64),
    Str(Cow<'a, str>),
    FixedOffset(DateTime<FixedOffset>),
    Utc(DateTime<Utc>),
    Blob(Cow<'a, [u8]>),
    Opt(Option<Box<SqlValue<'a>>>),
}

#[derive(Clone, Debug)]
pub struct SqlValueOwned(SqlValue<'static>);

#[derive(Clone, Debug)]
pub struct SqlValueRow<T> {
    pub row: HashMap<Arc<str>, T>,
}

impl Deref for SqlValueOwned {
    type Target = SqlValue<'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<SqlValue<'a>> for SqlValueOwned {
    fn from(value: SqlValue<'a>) -> Self {
        Self(value.live_static())
    }
}

impl From<Value> for SqlValueOwned {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => Self(SqlValue::Opt(None)),
            Value::Integer(v) => Self(SqlValue::I64(v)),
            Value::Real(v) => Self(SqlValue::F64(v)),
            Value::Text(v) => Self(SqlValue::Str(v.into())),
            Value::Blob(v) => Self(SqlValue::Blob(v.into())),
        }
    }
}

impl From<SqlValueOwned> for SqlValue<'static> {
    fn from(value: SqlValueOwned) -> Self {
        value.0
    }
}

impl<'a> Borrow<SqlValue<'a>> for SqlValueOwned {
    fn borrow(&self) -> &SqlValue<'a> {
        &self.0
    }
}

impl<'a> SqlValue<'a> {
    pub fn live_static(self) -> SqlValue<'static> {
        match self {
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
            SqlValue::F64(v) => SqlValue::F64(v),
            SqlValue::Blob(cow) => SqlValue::Blob(Cow::Owned(cow.to_vec())),
            SqlValue::Str(cow) => SqlValue::Str(Cow::Owned(cow.into_owned())),
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

impl<'a> From<bool> for SqlValue<'a> {
    fn from(val: bool) -> Self {
        SqlValue::Bool(val)
    }
}

impl<'a> From<DateTime<FixedOffset>> for SqlValue<'a> {
    fn from(val: DateTime<FixedOffset>) -> Self {
        SqlValue::FixedOffset(val)
    }
}

impl<'a> From<&'a DateTime<FixedOffset>> for SqlValue<'a> {
    fn from(val: &'a DateTime<FixedOffset>) -> Self {
        SqlValue::FixedOffset(*val)
    }
}

impl<'a> From<&'a DateTime<Utc>> for SqlValue<'a> {
    fn from(val: &'a DateTime<Utc>) -> Self {
        SqlValue::Utc(*val)
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

macro_rules! try_from_sql_value {
    ($tp:ty, $($variant:ident => $conv:expr),*) => {
        impl<'a> TryFrom<SqlValue<'a>> for $tp {
            type Error = ChinSqlError;

            fn try_from(value: SqlValue<'a>) -> Result<Self, Self::Error> {
                match value {
                    $(
                        SqlValue::$variant(v) => Ok($conv(v)?),
                    )*
                    _ => Err(ChinSqlError::TransformError(
                        concat!("Unable to transform to ", stringify!($tp)).to_owned(),
                    )),
                }
            }
        }

        impl<'a> TryFrom<SqlValue<'a>> for Option<$tp> {
            type Error = ChinSqlError;

            fn try_from(value: SqlValue<'a>) -> Result<Self, Self::Error> {
                match value {
                    SqlValue::Opt(v) => {
                        if let Some(v) = v {
                            match *v {
                                $(
                                    SqlValue::$variant(v) => Ok(Some($conv(v)?)),
                                )*
                                _ => Err(ChinSqlError::TransformError(
                                    concat!("Unable to transform ", stringify!($tp)).to_owned(),
                                )),
                            }
                        } else {
                            Ok(None)
                        }
                    },
                    $(
                        SqlValue::$variant(v) => Ok(Some($conv(v)?)),
                    )*
                    _ => Err(ChinSqlError::TransformError(
                        concat!("Unable to transform ", stringify!($tp)).to_owned(),
                    )),
                }
            }
        }
    };
}

try_from_sql_value!(DateTime<FixedOffset>,
    FixedOffset => |v: DateTime<FixedOffset>| Ok(v),
    I64 => |v: i64| Timestamptz::try_from(v).map(|tz| *tz),
    Str => |v: Cow<'a, str>|  DateTime::parse_from_str(&v, "%Y-%m-%dT%H:%M:%S%.9f %z").map_err(|err| ChinSqlError::TransformError(err.to_string()))
);

try_from_sql_value!(bool,
    Bool => |v: bool| Ok(v),
    I64 => |v: i64| Ok(v != 0)
);
try_from_sql_value!(i64, I64 => |v: i64| Ok(v));
try_from_sql_value!(i32, I32 => |v: i32| Ok(v));
try_from_sql_value!(f64, F64 => |v: f64|Ok(v));
try_from_sql_value!(Cow<'a, str>, Str => |v: Cow<'a, str>| Ok(v));

try_from_sql_value!(String,
    Str => |v: Cow<'a, str>| Ok(v.to_string())
);
