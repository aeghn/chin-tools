use std::borrow::Cow;

use crate::SqlValue;

pub enum PlaceHolderType {
    QustionMark,
    DollarNumber(i32),
}

impl PlaceHolderType {
    pub fn dollar_number() -> Self {
        PlaceHolderType::DollarNumber(0)
    }

    pub fn question_mark() -> Self {
        PlaceHolderType::QustionMark
    }

    pub fn next_ph(&mut self) -> String {
        match self {
            PlaceHolderType::QustionMark => "?".to_owned(),
            PlaceHolderType::DollarNumber(n) => {
                *n += 1;
                format!("${}", n)
            }
        }
    }
}

pub enum SegOrVal<'a> {
    Str(Cow<'a, str>),
    Val(SqlValue<'a>),
}

impl<'a> From<&'a str> for SegOrVal<'a> {
    fn from(value: &'a str) -> Self {
        Self::Str(Cow::Borrowed(value))
    }
}

impl<'a> From<String> for SegOrVal<'a> {
    fn from(value: String) -> Self {
        Self::Str(Cow::Owned(value))
    }
}

impl<'a> From<SqlValue<'a>> for SegOrVal<'a> {
    fn from(value: SqlValue<'a>) -> Self {
        Self::Val(value)
    }
}

impl<'a> SegOrVal<'a> {
    pub fn val<T: Into<SqlValue<'a>>>(val: T) -> Self {
        SegOrVal::Val(val.into())
    }
}
