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

use chin_tools_types::time_type::TID;
use chrono::{DateTime, FixedOffset, Utc};
use rusqlite::types::Value;
use sqlite::sqltype::Timestamptz;

use crate::ChinSqlError;

#[derive(Clone, Debug)]
pub enum RustFieldType {
    Bool,
    I8,
    I16,
    I32,
    I64,
    F64,
    Text,
    Blob,
    Timestamptz,
    Timestamp,
    Any,
}

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
    Null(RustFieldType),
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

impl<'a, T: Into<SqlValue<'a>>> From<T> for SqlValueOwned {
    fn from(value: T) -> Self {
        let value = value.into();
        Self(value.live_static())
    }
}

impl From<Value> for SqlValueOwned {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => Self(SqlValue::Null(RustFieldType::Any)),
            Value::Integer(v) => Self(SqlValue::I64(v)),
            Value::Real(v) => Self(SqlValue::F64(v)),
            Value::Text(v) => Self(SqlValue::Str(v.into())),
            Value::Blob(v) => Self(SqlValue::Blob(v.into())),
        }
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
            SqlValue::Null(logic_field_type) => SqlValue::Null(logic_field_type),
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

impl<'a> From<TID> for SqlValue<'a> {
    fn from(value: TID) -> Self {
        SqlValue::I64(value.into())
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

impl<'a> From<f64> for SqlValue<'a> {
    fn from(val: f64) -> Self {
        SqlValue::F64(val)
    }
}

macro_rules! try_from_sql_value {
    ($tp:ty, $rlt:expr, $($variant:ident => $conv:expr),*) => {
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
                    SqlValue::Null(_) => {
                        Ok(None)
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

        impl<'a> From<&$tp> for SqlValue<'a> {
            fn from(val: &$tp) -> Self {
                val.to_owned().into()
            }
        }

        impl<'a> From<Option<$tp>> for SqlValue<'a> {
            fn from(val: Option<$tp>) -> Self {
                match val {
                    Some(v) => v.into(),
                    None => SqlValue::Null($rlt),
                }
            }
        }

        impl<'a> From<Option<&$tp>> for SqlValue<'a> {
            fn from(val: Option<&$tp>) -> Self {
                match val {
                    Some(v) => v.to_owned().into(),
                    None => SqlValue::Null($rlt),
                }
            }
        }
    };
}

try_from_sql_value!(DateTime<FixedOffset>, RustFieldType::Timestamptz,
    FixedOffset => |v: DateTime<FixedOffset>| Ok(v),
    I64 => |v: i64| Timestamptz::try_from(v).map(|tz| *tz),
    Str => |v: Cow<'a, str>|  DateTime::parse_from_str(&v, "%Y-%m-%dT%H:%M:%S%.9f %z").map_err(|err| ChinSqlError::TransformError(err.to_string()))
);

try_from_sql_value!(bool, RustFieldType::Bool,
    Bool => |v: bool| Ok(v),
    I64 => |v: i64| Ok(v != 0)
);
try_from_sql_value!(i64, RustFieldType::I64, I64 => |v: i64| Ok(v));
try_from_sql_value!(i32, RustFieldType::I32, I32 => |v: i32| Ok(v));
try_from_sql_value!(f64, RustFieldType::F64, F64 => |v: f64|Ok(v));
try_from_sql_value!(Cow<'a, str>, RustFieldType::Text, Str => |v: Cow<'a, str>| Ok(v));
try_from_sql_value!(String, RustFieldType::Text,
    Str => |v: Cow<'a, str>| Ok(v.to_string())
);
try_from_sql_value!(TID, RustFieldType::I64, I64 => |v: i64|Ok(v.into()));
