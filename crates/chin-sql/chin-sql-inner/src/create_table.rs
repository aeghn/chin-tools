use crate::{IntoSqlSeg, LogicFieldType, SqlBuilder};

#[derive(Clone, Debug)]
pub struct CreateTableField {
    pub name: &'static str,
    pub kind: LogicFieldType,
    pub not_null: bool,
}

#[derive(Clone, Debug)]
pub struct CreateTableSql {
    pub table_name: &'static str,
    pub fields: &'static [CreateTableField],
    pub pkey: &'static [&'static str],
    pub unikeys: &'static [(&'static str, &'static [&'static str])],
    pub keys: &'static [(&'static str, &'static [&'static str])],
}

impl LogicFieldType {
    fn to_type(self, db_type: crate::DbType) -> String {
        match db_type {
            crate::DbType::Sqlite => match self {
                LogicFieldType::Bool => "INTEGER".into(),
                LogicFieldType::I8 => "INTEGER".into(),
                LogicFieldType::I16 => "INTEGER".into(),
                LogicFieldType::I32 => "INTEGER".into(),
                LogicFieldType::I64 => "INTEGER".into(),
                LogicFieldType::F64 => "REAL".into(),
                LogicFieldType::Varchar(_) => "TEXT".into(),
                LogicFieldType::Text => "TEXT".into(),
                LogicFieldType::Blob => "BLOB".into(),
                LogicFieldType::Timestamptz => "INTEGER".into(),
                LogicFieldType::Timestamp => "INTEGER".into(),
            },
            crate::DbType::Postgres => match self {
                LogicFieldType::Bool => "BOOL".into(),
                LogicFieldType::I8 => "CHAR".into(),
                LogicFieldType::I16 => "INT2".into(),
                LogicFieldType::I32 => "INT4".into(),
                LogicFieldType::I64 => "INT8".into(),
                LogicFieldType::F64 => "FLOAT8".into(),
                LogicFieldType::Varchar(len) => format!("Varchar({len})"),
                LogicFieldType::Text => "TEXT".into(),
                LogicFieldType::Blob => "BLOB".into(),
                LogicFieldType::Timestamptz => "TIMESTAMPTZ".into(),
                LogicFieldType::Timestamp => "TIMESTAMP".into(),
            },
        }
    }
}

impl CreateTableSql {
    pub fn sqls(&self, db_type: crate::DbType) -> Result<Vec<String>, crate::ChinSqlError> {
        let mut sr = SqlBuilder::new()
            .sov("create table if not exists")
            .sov(self.table_name)
            .sov("(");

        let columns: Vec<String> = self
            .fields
            .iter()
            .map(|f| {
                format!(
                    "{} {} {}",
                    f.name,
                    f.kind.to_type(db_type),
                    if f.not_null { "not null" } else { "" }
                )
            })
            .collect();
        sr = sr.sov(columns.join(", "));
        if !self.pkey.is_empty() {
            sr = sr
                .sov(", ")
                .sov("primary key (")
                .sov(self.pkey.join(","))
                .sov(")");
        }
        sr = sr.sov(")");

        let mut result = vec![];
        let ct = sr
            .into_sql_seg2(db_type, &mut crate::PlaceHolderType::QustionMark)?
            .seg;
        result.push(ct);
        for (key, fields) in self.unikeys {
            result.push(format!(
                "create unique index if not exists {}_{} on {}({})",
                self.table_name,
                key,
                self.table_name,
                fields.join(",")
            ));
        }

        for (key, fields) in self.keys {
            result.push(format!(
                "create index if not exists {}_{} on {}({})",
                self.table_name,
                key,
                self.table_name,
                fields.join(",")
            ));
        }

        Ok(result)
    }
}
