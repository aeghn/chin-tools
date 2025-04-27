use std::borrow::Cow;

use super::{SqlSeg, place_hoder::PlaceHolderType, sql_value::SqlValue};

pub enum WhereConjOp {
    And,
    Or,
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

    pub fn ilike<T: AsRef<str>>(key: &'a str, v: T) -> Self {
        Self::Compare {
            key,
            operator: "ilike",
            value: format!("%{}%", v.as_ref()).into(),
        }
    }

    pub fn is_null(key: &'a str) -> Self {
        Self::compare_str(key, "is", "null")
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
        let s = Self::In(key, values.into_iter().map(|e| e.into()).collect());
        s
    }

    pub fn none() -> Self {
        Self::None
    }

    pub fn build(self, value_type: &mut PlaceHolderType) -> Option<SqlSeg<'a>> {
        let mut seg = String::new();
        let mut values: Vec<SqlValue<'a>> = Vec::new();

        match self {
            Wheres::Conj(op, fs) => {
                let vs: Vec<String> = fs
                    .into_iter()
                    .filter_map(|e| {
                        e.build(value_type).map(|ss| {
                            values.extend(ss.values);
                            ss.seg
                        })
                    })
                    .collect();
                if vs.len() == 0 {
                    return None;
                }
                let op = match op {
                    WhereConjOp::And => " and ",
                    WhereConjOp::Or => " or ",
                };

                seg.push_str(vs.join(op).as_str())
            }
            Wheres::In(key, fs) => {
                seg.push_str(key);
                seg.push_str(" in (");
                let vs = fs
                    .iter()
                    .map(|_| value_type.next())
                    .collect::<Vec<String>>();
                if fs.len() == 0 {
                    return None;
                }
                seg.push_str(vs.join(",").as_str());

                seg.push_str(")");
                values.extend(fs.into_iter())
            }
            Wheres::Not(fs) => {
                seg.push_str(" not ( ");
                if let Some(ss) = fs.build(value_type) {
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
                seg.push_str(" ");
                seg.push_str(operator);
                seg.push_str(" ");

                seg.push_str(&value_type.next());
                values.push(value);
            }
            Wheres::Raw(cow) => {
                if !seg.ends_with(" ") {
                    seg.push(' ');
                }
                seg.push_str(cow.as_ref());

                seg.push(' ');
            }
        }

        Some(SqlSeg { seg, values })
    }
}
