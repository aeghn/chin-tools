use std::borrow::Cow;

use crate::{DbType, PlaceHolderType, SegOrVal, SqlSeg};

use super::sql_value::SqlValue;

pub enum WhereConjOp {
    And,
    Or,
}

pub enum ILikeType {
    Original,
    RightFuzzy,
    LeftFuzzy,
    Fuzzy,
}

pub struct FilterCount {
    pub check_filter_count: bool,
    pub filter_count: usize,
}

impl Default for FilterCount {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterCount {
    pub fn new() -> Self {
        Self {
            check_filter_count: true,
            filter_count: 0,
        }
    }

    pub fn increament(self) -> Self {
        Self {
            filter_count: self.filter_count + 1,
            ..self
        }
    }
    pub fn check(&self) -> bool {
        self.check_filter_count && self.filter_count > 0
    }
}

pub enum Wheres<'a> {
    Conj(WhereConjOp, Vec<Wheres<'a>>),
    In(&'a str, Vec<SqlValue<'a>>),
    Not(Box<Wheres<'a>>),
    Compare {
        key: &'a str,
        operator: &'a str,
        value: SqlValue<'a>,
    }, // key, operator, value
    Raw(Cow<'a, str>),
    SOV(Vec<SegOrVal<'a>>),
    IIike {
        key: &'a str,
        value: String,
    },
    None,
}

impl<'a> Wheres<'a> {
    pub fn equal<'b: 'a, T: Into<SqlValue<'a>>>(key: &'b str, v: T) -> Self {
        Self::Compare {
            key,
            operator: "=",
            value: v.into(),
        }
    }

    pub fn ilike<T: AsRef<str>>(key: &'a str, v: T, exact: ILikeType) -> Self {
        let s = v.as_ref();
        if s.is_empty() {
            return Wheres::None;
        }
        Self::IIike {
            key,
            value: match exact {
                ILikeType::Original => v.as_ref().into(),
                ILikeType::RightFuzzy => format!("{}%", v.as_ref()),
                ILikeType::LeftFuzzy => format!("%{}", v.as_ref()),
                ILikeType::Fuzzy => format!("%{}%", v.as_ref()),
            },
        }
    }
    pub fn is_null(key: &'a str) -> Self {
        Self::compare_str(key, "is", "null")
    }

    pub fn is_not_null(key: &'a str) -> Self {
        Self::compare_str(key, "is not", "null")
    }

    pub fn compare<'b: 'a, T: Into<SqlValue<'a>>>(key: &'b str, operator: &'b str, v: T) -> Self {
        Self::Compare {
            key,
            operator,
            value: v.into(),
        }
    }

    pub fn compare_str<T: AsRef<str>>(key: &'a str, operator: &'a str, v: T) -> Self {
        Self::Raw(Cow::Owned(format!("{} {} {}", key, operator, v.as_ref())))
    }

    pub fn if_some<T, F>(original: Option<T>, map: F) -> Self
    where
        F: FnOnce(T) -> Self,
    {
        match original {
            Some(t) => map(t),
            None => Wheres::None,
        }
    }

    pub fn not<T: Into<Wheres<'a>>>(values: T) -> Self {
        Self::Not(Box::new(values.into()))
    }

    pub fn and<T: Into<Vec<Wheres<'a>>>>(values: T) -> Self {
        Self::Conj(WhereConjOp::And, values.into())
    }

    pub fn or<T: Into<Vec<Wheres<'a>>>>(values: T) -> Self {
        Self::Conj(WhereConjOp::Or, values.into())
    }

    pub fn transform<T, F>(original: T, map: F) -> Self
    where
        F: FnOnce(T) -> Self,
    {
        map(original)
    }

    pub fn r#in<T: Into<SqlValue<'a>>>(key: &'a str, values: Vec<T>) -> Self {
        Self::In(key, values.into_iter().map(|e| e.into()).collect())
    }

    pub fn none() -> Self {
        Self::None
    }

    pub fn of<T: Into<SegOrVal<'a>>>(sovs: Vec<T>) -> Self {
        Self::SOV(sovs.into_iter().map(|e| e.into()).collect())
    }

    pub fn build(self, db_type: DbType, value_type: &mut PlaceHolderType) -> Option<SqlSeg<'a>> {
        let mut seg = String::new();
        let mut values: Vec<SqlValue<'a>> = Vec::new();

        match self {
            Wheres::Conj(op, fs) => {
                let vs: Vec<String> = fs
                    .into_iter()
                    .filter_map(|e| {
                        e.build(db_type, value_type).map(|ss| {
                            values.extend(ss.values);
                            ss.seg
                        })
                    })
                    .collect();
                if vs.is_empty() {
                    return None;
                }
                let op = match op {
                    WhereConjOp::And => " and ",
                    WhereConjOp::Or => " or ",
                };

                seg.push_str(vs.join(op).as_str())
            }
            Wheres::In(key, fs) => {
                log::info!("print: {:?}, {:?}", key, fs);
                seg.push_str(key);
                seg.push_str(" in (");
                let vs = fs
                    .iter()
                    .map(|_| value_type.next_ph())
                    .collect::<Vec<String>>();
                seg.push_str(vs.join(",").as_str());

                seg.push(')');
                values.extend(fs)
            }
            Wheres::Not(fs) => {
                seg.push_str(" not ( ");
                if let Some(ss) = fs.build(db_type, value_type) {
                    seg.push_str(&ss.seg);
                    seg.push(')');

                    values.extend(ss.values);
                } else {
                    return None;
                }
            }
            Wheres::None => {
                return None;
            }
            Wheres::Compare {
                key,
                operator,
                value,
            } => {
                seg.push_str(key);
                seg.push(' ');
                seg.push_str(operator);
                seg.push(' ');

                seg.push_str(&value_type.next_ph());
                values.push(value);
            }
            Wheres::Raw(cow) => {
                if !seg.ends_with(" ") {
                    seg.push(' ');
                }
                seg.push_str(cow.as_ref());

                seg.push(' ');
            }
            Wheres::SOV(seg_or_vals) => {
                for sov in seg_or_vals {
                    match sov {
                        SegOrVal::Str(cow) => {
                            seg.push_str(&cow);
                        }
                        SegOrVal::Val(sql_value) => {
                            seg.push_str(&value_type.next_ph());
                            values.push(sql_value);
                        }
                    }
                }
            }
            Wheres::IIike { key, value } => {
                let ilike = match db_type {
                    DbType::Sqlite => Self::Compare {
                        key,
                        operator: "like",
                        value: value.into(),
                    },
                    DbType::Postgres => Self::Compare {
                        key,
                        operator: "ilike",
                        value: value.into(),
                    },
                };
                let s = ilike.build(db_type, value_type);
                if let Some(SqlSeg { seg: s, values: v }) = s {
                    seg.push_str(s.as_str());
                    values.extend(v);
                }
            }
        }

        Some(SqlSeg::of(seg, values))
    }
}
