mod place_hoder;
mod sql_builder;
mod sql_inserter;
mod sql_updater;
mod sql_value;
mod wheres;

pub use chin_sql_derive::GenerateTableSql;
pub use chin_tools_base::DbType;
pub use place_hoder::*;
pub use sql_builder::*;
pub use sql_inserter::*;
pub use sql_updater::*;
pub use sql_value::*;
pub use wheres::*;

use thiserror::Error;

#[derive(Clone, Debug)]
pub struct SqlSeg<'a> {
    pub seg: String,
    pub values: Vec<SqlValue<'a>>,
}

pub trait IntoSqlSeg<'a> {
    fn into_sql_seg(self, db_type: DbType) -> Result<SqlSeg<'a>, ChinSqlError>;
}

#[derive(Error, Debug)]
pub enum ChinSqlError {
    #[error("Cannot build sql")]
    BuilderSqlError,
}
