use crate::{ChinSqlError, IntoSqlSeg, SegOrVal, SqlSeg};

use super::{place_hoder::PlaceHolderType, sql_value::SqlValue, wheres::Wheres};

pub trait CustomSqlSeg<'a>: Send {
    fn build(&self, value_type: &mut PlaceHolderType) -> Option<SqlSeg<'a>>;
}

pub enum SqlSegType<'a> {
    Where(Wheres<'a>),
    Comma(Vec<&'a str>),
    Raw(&'a str),
    SegOrVal(SegOrVal<'a>),
    RawOwned(String),
    Custom(Box<dyn CustomSqlSeg<'a>>),
    Sub {
        alias: &'a str,
        query: SqlSegBuilder<'a>,
    },
}

pub struct SqlSegBuilder<'a> {
    segs: Vec<SqlSegType<'a>>,
}

impl<'a> SqlSegBuilder<'a> {
    pub fn new() -> Self {
        Self { segs: vec![] }
    }

    pub fn seg(mut self, seg: SqlSegType<'a>) -> Self {
        self.segs.push(seg);
        self
    }

    pub fn raw(mut self, seg: &'a str) -> Self {
        self.segs.push(SqlSegType::Raw(seg));
        self
    }

    pub fn sov<T: Into<SqlValue<'a>>>(mut self, sov: SegOrVal<'a>) -> Self {
        self.segs.push(SqlSegType::SegOrVal(sov));
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
        self.segs.push(SqlSegType::Where(wheres.into()));
        self
    }

    pub fn comma(mut self, values: Vec<&'a str>) -> Self {
        self.segs.push(SqlSegType::Comma(values));
        self
    }

    pub fn sub(mut self, alias: &'a str, query: SqlSegBuilder<'a>) -> Self {
        self.segs.push(SqlSegType::Sub { alias, query });
        self
    }

    pub fn custom<T: CustomSqlSeg<'a> + 'static>(mut self, custom: T) -> Self {
        self.segs.push(SqlSegType::Custom(Box::new(custom)));
        self
    }
}

pub struct LimitOffset {
    limit: u64,
    offset: Option<u64>,
}

impl LimitOffset {
    pub fn new(limit: u64) -> Self {
        Self {
            limit,
            offset: None,
        }
    }

    pub fn offset(mut self, offset: u64) -> Self {
        self.offset.replace(offset);

        self
    }

    pub fn offset_if_some(mut self, offset: Option<u64>) -> Self {
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

impl<'a> IntoSqlSeg<'a> for SqlSegBuilder<'a> {
    fn into_sql_seg2(
        self,
        db_type: chin_tools_base::DbType,
        pht: &mut PlaceHolderType,
    ) -> Result<SqlSeg<'a>, ChinSqlError> {
        if self.segs.is_empty() {
            Err(ChinSqlError::BuilderSqlError)?
        }

        let mut sb = String::new();
        let mut values: Vec<SqlValue<'a>> = Vec::new();

        for seg in self.segs {
            match seg {
                SqlSegType::Where(wr) => {
                    if let Some(ss) = wr.build(db_type, pht) {
                        sb.push_str(" where ");
                        sb.push_str(&ss.seg);
                        values.extend(ss.values)
                    }
                }
                SqlSegType::Comma(vs) => {
                    sb.push_str(vs.join(", ").as_str());
                }
                SqlSegType::Raw(raw) => {
                    sb.push_str(&raw);
                }
                SqlSegType::Custom(custom) => {
                    if let Some(cs) = custom.build(pht) {
                        sb.push_str(&cs.seg);
                        values.extend(cs.values)
                    }
                }
                SqlSegType::Sub { alias, query } => {
                    if let Ok(s) = query.into_sql_seg2(db_type, pht) {
                        sb.push_str(" (");
                        sb.push_str(&s.seg);
                        sb.push_str(") ");
                        sb.push_str(&alias);
                        values.extend(s.values);
                    }
                }
                SqlSegType::RawOwned(raw) => {
                    sb.push_str(raw.as_str());
                }
                SqlSegType::SegOrVal(sql_seg) => match sql_seg {
                    SegOrVal::Str(s) => {
                        sb.push_str(&s);
                    }
                    SegOrVal::Val(val) => {
                        sb.push_str(&pht.next());
                        values.push(val);
                    }
                },
            };
            if !sb.ends_with(" ") {
                sb.push(' ');
            }
        }

        Ok(SqlSeg::of(sb, values))
    }
}
