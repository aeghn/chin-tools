use std::ops::Deref;

use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[cfg(feature = "postgres")]
use postgres_types::{to_sql_checked, FromSql, ToSql};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct SharedStr(SmolStr);

impl Deref for SharedStr {
    type Target = SmolStr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for SharedStr {
    fn from(value: String) -> Self {
        Self(SmolStr::from(value))
    }
}

impl From<&'_ str> for SharedStr {
    fn from(value: &'_ str) -> Self {
        Self(SmolStr::from(value))
    }
}

impl AsRef<str> for SharedStr {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl SharedStr {
    pub fn new(s: impl AsRef<str>) -> Self {
        Self(SmolStr::new(s))
    }
}

impl AsRef<[u8]> for SharedStr {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

#[cfg(feature = "postgres")]
impl ToSql for SharedStr {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut bytes::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        self.to_string().to_sql(ty, out)
    }

    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        <String as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}

#[cfg(feature = "postgres")]
impl<'a> FromSql<'a> for SharedStr {
    fn from_sql(
        ty: &postgres_types::Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        <&str as postgres_types::FromSql>::from_sql(ty, raw).and_then(|s| Ok(SharedStr::new(s)))
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        <&str as FromSql>::accepts(ty)
    }
}
