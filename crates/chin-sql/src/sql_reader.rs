use crate::{ChinSqlError, IntoSqlSeg, SegOrVal, SqlSeg};

use super::{place_hoder::PlaceHolderType, sql_value::SqlValue, wheres::Wheres};

pub trait CustomSqlSeg<'a>: Send {
    fn build(&self, value_type: &mut PlaceHolderType) -> Option<SqlSeg<'a>>;
}

enum SqlReaderSeg<'a> {
    Where(Wheres<'a>),
    LimitOffset(LimitOffset),
    Comma(Vec<&'a str>),
    Raw(&'a str),
    SegOrVal(SegOrVal<'a>),
    RawOwned(String),
    Custom(Box<dyn CustomSqlSeg<'a>>),
    Sub {
        alias: &'a str,
        query: SqlReader<'a>,
    },
}

pub struct SqlReader<'a> {
    segs: Vec<SqlReaderSeg<'a>>,
}

impl<'a> Default for SqlReader<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> SqlReader<'a> {
    pub fn new() -> Self {
        Self { segs: vec![] }
    }

    pub fn read(table_name: &str, fields: &[&str]) -> Self {
        Self {
            segs: vec![SqlReaderSeg::RawOwned(format!(
                "select {} from {} ",
                fields.join(", "),
                table_name
            ))],
        }
    }

    pub fn read_all(table_name: &str) -> Self {
        Self {
            segs: vec![SqlReaderSeg::RawOwned(format!(
                "select * from {} ",
                table_name
            ))],
        }
    }

    pub fn raw(mut self, seg: &'a str) -> Self {
        self.segs.push(SqlReaderSeg::Raw(seg));
        self
    }

    pub fn sov<T: Into<SegOrVal<'a>>>(mut self, sov: T) -> Self {
        self.segs.push(SqlReaderSeg::SegOrVal(sov.into()));
        self
    }

    pub fn some_then<T, F>(self, cond: Option<T>, trans: F) -> Self
    where
        F: FnOnce(T, Self) -> Self,
    {
        if let Some(t) = cond {
            trans(t, self)
        } else {
            self
        }
    }

    pub fn r#where<T: Into<Wheres<'a>>>(mut self, wheres: T) -> Self {
        self.segs.push(SqlReaderSeg::Where(wheres.into()));
        self
    }

    pub fn comma(mut self, values: Vec<&'a str>) -> Self {
        self.segs.push(SqlReaderSeg::Comma(values));
        self
    }

    pub fn sub(mut self, alias: &'a str, query: SqlReader<'a>) -> Self {
        self.segs.push(SqlReaderSeg::Sub { alias, query });
        self
    }

    pub fn custom<T: CustomSqlSeg<'a> + 'static>(mut self, custom: T) -> Self {
        self.segs.push(SqlReaderSeg::Custom(Box::new(custom)));
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.segs
            .push(SqlReaderSeg::LimitOffset(LimitOffset::new(limit)));
        self
    }

    pub fn limit_offset(mut self, limit: LimitOffset) -> Self {
        self.segs.push(SqlReaderSeg::LimitOffset(limit));
        self
    }
}

pub struct LimitOffset {
    limit: usize,
    offset: Option<usize>,
}

impl LimitOffset {
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            offset: None,
        }
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset.replace(offset);

        self
    }

    pub fn offset_if_some(mut self, offset: Option<usize>) -> Self {
        self.offset = offset;

        self
    }

    pub fn to_box(self) -> Box<dyn CustomSqlSeg<'static>> {
        Box::new(self)
    }
}

impl<'a> CustomSqlSeg<'a> for LimitOffset {
    fn build(&self, _: &mut PlaceHolderType) -> Option<SqlSeg<'a>> {
        match self.offset {
            Some(v) => Some(SqlSeg::of(
                format!("limit {} offset {}", self.limit, v),
                vec![],
            )),
            None => Some(SqlSeg::of(format!("limit {}", self.limit), vec![])),
        }
    }
}

impl<'a> IntoSqlSeg<'a> for SqlReader<'a> {
    fn into_sql_seg2(
        self,
        db_type: chin_tools_types::DbType,
        pht: &mut PlaceHolderType,
    ) -> Result<SqlSeg<'a>, ChinSqlError> {
        if self.segs.is_empty() {
            Err(ChinSqlError::BuilderSqlError)?
        }

        let mut sb = String::new();
        let mut values: Vec<SqlValue<'a>> = Vec::new();

        for seg in self.segs {
            match seg {
                SqlReaderSeg::Where(wr) => {
                    if let Some(ss) = wr.build(db_type, pht) {
                        sb.push_str(" where ");
                        sb.push_str(&ss.seg);
                        values.extend(ss.values)
                    }
                }
                SqlReaderSeg::Comma(vs) => {
                    sb.push_str(vs.join(", ").as_str());
                }
                SqlReaderSeg::Raw(raw) => {
                    sb.push_str(raw);
                }
                SqlReaderSeg::Custom(custom) => {
                    if let Some(cs) = custom.build(pht) {
                        sb.push_str(&cs.seg);
                        values.extend(cs.values)
                    }
                }
                SqlReaderSeg::Sub { alias, query } => {
                    if let Ok(s) = query.into_sql_seg2(db_type, pht) {
                        sb.push_str(" (");
                        sb.push_str(&s.seg);
                        sb.push_str(") ");
                        sb.push_str(alias);
                        values.extend(s.values);
                    }
                }
                SqlReaderSeg::RawOwned(raw) => {
                    sb.push_str(raw.as_str());
                }
                SqlReaderSeg::SegOrVal(sql_seg) => match sql_seg {
                    SegOrVal::Str(s) => {
                        sb.push_str(&s);
                    }
                    SegOrVal::Val(val) => {
                        sb.push_str(&pht.next());
                        values.push(val);
                    }
                },
                SqlReaderSeg::LimitOffset(limit_offset) => {
                    let SqlSeg { seg, values: vs } =
                        limit_offset.build(pht).ok_or(ChinSqlError::TransformError(
                            "Unable convert limit offset to sql seg.".to_owned(),
                        ))?;
                    sb.push_str(&seg);
                    values.extend(vs);
                }
            };
            if !sb.ends_with(" ") {
                sb.push(' ');
            }
        }

        Ok(SqlSeg::of(sb, values))
    }
}
