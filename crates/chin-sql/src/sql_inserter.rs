use crate::{ChinSqlError, PlaceHolderType, SqlSegBuilder};

use super::{SqlSeg, sql_value::SqlValue};

pub struct SqlInserter<'a> {
    table: &'a str,
    fields: Vec<(&'a str, SqlValue<'a>)>,
    extra: Vec<(&'a str, SqlValue<'a>)>,
}

impl<'a> SqlInserter<'a> {
    pub fn new(table: &'a str) -> Self {
        SqlInserter {
            table: &table,
            fields: vec![],
            extra: vec![],
        }
    }

    pub fn fields<T: Into<SqlValue<'a>>>(mut self, key: &'a str, value: T) -> Self {
        self.fields.push((key, value.into()));

        self
    }

    pub fn raw<T: Into<SqlValue<'a>>>(mut self, key: &'a str, value: Option<T>) -> Self {
        if let Some(v) = value {
            self.extra.push((key, v.into()));
        }

        self
    }
    pub fn build(self, pht: PlaceHolderType) -> Result<SqlSeg<'a>, ChinSqlError> {
        todo!()
    }
}
