use chin_tools_types::SharedStr;
use serde::{Deserialize, Deserializer, Serialize, de};

use crate::ChinSqlError;

#[derive(Clone, Debug)]
pub struct Varchar<const LIMIT: usize>(SharedStr);

#[derive(Clone, Debug)]
pub struct Text(SharedStr);

impl<const LIMIT: usize> Serialize for Varchar<LIMIT> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de, const LIMIT: usize> Deserialize<'de> for Varchar<LIMIT> {
    fn deserialize<D>(deserializer: D) -> Result<Varchar<LIMIT>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let o: String = String::deserialize(deserializer)?;
        if o.len() > LIMIT {
            return Err(de::Error::custom(format!(
                "out of ranch: {} > {}",
                o.len(),
                LIMIT
            )));
        }
        Ok(Self(o.into()))
    }
}

impl<const LIMIT: usize> TryFrom<String> for Varchar<LIMIT> {
    type Error = ChinSqlError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() > LIMIT {
            return Err(ChinSqlError::TransformError(format!(
                "out of ranch: {} > {}",
                value.len(),
                LIMIT
            )));
        }

        Ok(Self(SharedStr::from(value)))
    }
}

impl<const LIMIT: usize> From<Varchar<LIMIT>> for String {
    fn from(value: Varchar<LIMIT>) -> Self {
        value.0.to_string()
    }
}


