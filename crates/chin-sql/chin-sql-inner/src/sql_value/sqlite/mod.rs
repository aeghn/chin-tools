use sqltype::Timestamptz;

use rusqlite::{ToSql, types::Value};

use super::{SqlValue, SqlValueOwned};

pub mod sqltype;

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
            SqlValue::F64(v) => v.to_sql(),
            SqlValue::Blob(cow) => cow.to_sql(),
            SqlValue::Null(rust_field_type) => None::<String>.to_sql(),
        }
    }
}

impl ToSql for SqlValueOwned {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl<'a> From<SqlValue<'a>> for Value {
    fn from(value: SqlValue<'a>) -> Self {
        match value {
            SqlValue::Bool(v) => Value::from(v),
            SqlValue::I8(v) => Value::from(v),
            SqlValue::I16(v) => Value::from(v),
            SqlValue::I32(v) => Value::from(v),
            SqlValue::I64(v) => Value::from(v),
            SqlValue::F64(v) => Value::from(v),
            SqlValue::Str(v) => Value::from(v.to_string()),
            SqlValue::FixedOffset(date_time) => {
                        Value::from(i64::from(Timestamptz::from(date_time)))
                    }
            SqlValue::Utc(date_time) => Value::from(i64::from(Timestamptz::from(date_time))),
            SqlValue::Blob(v) => Value::from(v.to_vec()),
            SqlValue::Null(_) => Value::Null,
        }
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
