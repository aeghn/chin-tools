use chrono::DateTime;
use sqltype::Timestamptz;

use std::str;

use rusqlite::{
    ToSql,
    types::{FromSql, FromSqlResult},
};

use super::{SqlValue, SqlValueOwned};

pub mod sqltype;

impl ToSql for Timestamptz {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Owned(
            rusqlite::types::Value::Text(self.0.format("%Y-%m-%dT%H:%M:%S%.9f %z").to_string()),
        ))
    }
}

impl FromSql for Timestamptz {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Text(items) => {
                match str::from_utf8(items)
                    .map(|e| DateTime::parse_from_str(e, "%Y-%m-%dT%H:%M:%S%.9f %z"))
                {
                    Ok(Ok(dt)) => Ok(Timestamptz(dt)),
                    Ok(Err(err)) => {
                        FromSqlResult::Err(rusqlite::types::FromSqlError::Other(Box::new(err)))
                    }
                    Err(err) => {
                        FromSqlResult::Err(rusqlite::types::FromSqlError::Other(Box::new(err)))
                    }
                }
            }
            _ => FromSqlResult::Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl<'a> ToSql for SqlValue<'a> {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        match self {
            SqlValue::I8(v) => v.to_sql(),
            SqlValue::I16(v) => v.to_sql(),
            SqlValue::I32(v) => v.to_sql(),
            SqlValue::I64(v) => v.to_sql(),
            SqlValue::Str(v) => v.to_sql(),
            SqlValue::FixedOffset(v) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Integer(i64::from(Timestamptz::from(*v))),
            )),
            SqlValue::Utc(v) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Integer(i64::from(Timestamptz::from(*v))),
            )),
            SqlValue::Bool(v) => v.to_sql(),
            SqlValue::Opt(v) => match v {
                Some(v) => v.as_ref().to_sql(),
                None => None::<String>.to_sql(),
            },
            SqlValue::F64(v) => v.to_sql(),
            SqlValue::Blob(cow) => cow.to_sql(),
        }
    }
}

impl ToSql for SqlValueOwned {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Local};

    use crate::sql_value::sqlite::sqltype::Timestamptz;


    #[test]
    fn test_convert() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute("create table ttime (t text not null)", [])
            .unwrap();
        conn.execute(
            "insert into ttime values(?)",
            [i64::from(Timestamptz::from(Local::now().fixed_offset()))],
        )
        .unwrap();
        let time = conn
            .query_row("select * from ttime", [], |e| {
                let t: String = e.get("t").unwrap();
                Ok(t)
            })
            .unwrap();

        println!("{}", time);
        println!(
            "{:?}",
            DateTime::parse_from_str(&time, "%Y-%m-%dT%H:%M:%S%.f %z").unwrap()
        );
    }
}
