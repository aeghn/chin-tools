use crate::{ChinSqlError, IntoSqlSeg};

use super::{SqlSeg, place_hoder::PlaceHolderType, sql_value::SqlValue, wheres::Wheres};

pub struct SqlDeleter<'a> {
    table: &'a str,
    wheres: Wheres<'a>,
}

impl<'a> SqlDeleter<'a> {
    pub fn new(table: &'a str) -> Self {
        SqlDeleter {
            table: &table,
            wheres: Wheres::and([]),
        }
    }

    pub fn r#where(mut self, wheres: Wheres<'a>) -> Self {
        self.wheres = wheres;
        self
    }
}

impl<'a> IntoSqlSeg<'a> for SqlDeleter<'a> {
    fn into_sql_seg2(
        self,
        db_type: chin_tools_base::DbType,
        pht: &mut PlaceHolderType,
    ) -> Result<SqlSeg<'a>, ChinSqlError> {
        let mut sb = String::new();
        let mut values: Vec<SqlValue<'a>> = Vec::new();

        sb.push_str("delete from ");
        sb.push_str(self.table);

        if let Some(filters) = self.wheres.build( db_type, pht) {
            sb.push_str(" where ");
            sb.push_str(filters.seg.as_str());

            values.extend(filters.values);
        }

        Ok(SqlSeg::of(sb, values))
    }
}
