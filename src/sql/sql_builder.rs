use super::{place_hoder::PlaceHolderType, sql_value::SqlValue, wheres::Wheres};

pub struct SqlSeg<'a> {
    pub seg: String,
    pub values: Vec<SqlValue<'a>>,
}

pub trait CustomSqlSeg<'a> {
    fn build(&self, value_type: &mut PlaceHolderType) -> Option<SqlSeg<'a>>;
}

pub enum SqlSegType<'a> {
    Where(Wheres<'a>),
    Comma(Vec<&'a str>),
    Raw(&'a str),
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
    pub fn raw_owned(mut self, seg: String) -> Self {
        self.segs.push(SqlSegType::RawOwned(seg));
        self
    }

    pub fn r#where(mut self, wheres: Wheres<'a>) -> Self {
        self.segs.push(SqlSegType::Where(wheres));
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

    pub fn custom(mut self, custom: Box<dyn CustomSqlSeg<'a>>) -> Self {
        self.segs.push(SqlSegType::Custom(custom));
        self
    }

    pub fn build(self, value_type: &mut PlaceHolderType) -> Option<SqlSeg<'a>> {
        if self.segs.is_empty() {
            return None;
        }

        let mut sb = String::new();
        let mut values: Vec<SqlValue<'a>> = Vec::new();

        for seg in self.segs {
            match seg {
                SqlSegType::Where(wr) => {
                    if let Some(ss) = wr.build(value_type) {
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
                    if let Some(cs) = custom.build(value_type) {
                        sb.push_str(&cs.seg);
                        values.extend(cs.values)
                    }
                }
                SqlSegType::Sub { alias, query } => {
                    if let Some(s) = query.build(value_type) {
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
            };
            if !sb.ends_with(" ") {
                sb.push(' ');
            }
        }

        Some(SqlSeg { seg: sb, values })
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
            Some(v) => Some(SqlSeg {
                seg: format!("limit {} offset {}", self.limit, v),
                values: vec![],
            }),
            None => Some(SqlSeg {
                seg: format!("limit {}", self.limit),
                values: vec![],
            }),
        }
    }
}
