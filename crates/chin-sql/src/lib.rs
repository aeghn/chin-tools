mod place_hoder;
mod sql_reader;
mod sql_deleter;
mod sql_inserter;
mod sql_updater;
mod sql_value;
mod wheres;

pub use chin_sql_derive::GenerateTableSql;
pub use chin_tools_base::DbType;
pub use place_hoder::*;
pub use sql_reader::*;
pub use sql_deleter::*;
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

impl<'a> SqlSeg<'a> {
    pub fn of<S: Into<String>>(seg: S, values: Vec<SqlValue<'a>>) -> Self {
        let seg = seg.into();
        Self { seg, values }
    }
}

pub trait IntoSqlSeg<'a>: Send {
    fn into_sql_seg(self, db_type: DbType) -> Result<SqlSeg<'a>, ChinSqlError>
    where
        Self: Sized,
    {
        match db_type {
            DbType::Sqlite => self.into_sql_seg2(db_type, &mut PlaceHolderType::QustionMark),
            DbType::Postgres => self.into_sql_seg2(db_type, &mut PlaceHolderType::DollarNumber(0)),
        }
    }

    fn into_sql_seg2(
        self,
        db_type: DbType,
        pht: &mut PlaceHolderType,
    ) -> Result<SqlSeg<'a>, ChinSqlError>;
}

impl<'a> IntoSqlSeg<'a> for SqlSeg<'a> {
    fn into_sql_seg2(
        self,
        _: DbType,
        _: &mut PlaceHolderType,
    ) -> Result<SqlSeg<'a>, ChinSqlError> {
        Ok(self)
    }
}

impl<'a> IntoSqlSeg<'a> for &'a str {
    fn into_sql_seg2(
        self,
        _: DbType,
        _: &mut PlaceHolderType,
    ) -> Result<SqlSeg<'a>, ChinSqlError> {
        Ok(SqlSeg::of(self, Vec::with_capacity(0)))
    }
}

impl<'a> IntoSqlSeg<'a> for String {
    fn into_sql_seg2(
        self,
        _: DbType,
        _: &mut PlaceHolderType,
    ) -> Result<SqlSeg<'a>, ChinSqlError> {
        Ok(SqlSeg::of(self, Vec::with_capacity(0)))
    }
}

#[derive(Error, Debug)]
pub enum ChinSqlError {
    #[error("Cannot build sql")]
    BuilderSqlError,
    #[error("Cannot map to struct {0}")]
    TransformError(String),
    #[error("Cannot build sql {0}")]
    FilterBuildError(String),
    
}
