#[cfg(feature = "postgres")]
use postgres_types::{FromSql, ToSql, to_sql_checked};

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
