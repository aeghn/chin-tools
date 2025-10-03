use std::{borrow::Cow, marker::PhantomData};

use crate::{str_type::Text, ILikeType, SqlBuilder, SqlValue, Wheres};

pub trait SqlTable<'a> {
    fn table_expr(&self) -> SqlBuilder<'a>;
    fn alias(&self) -> &'a str;
}

pub struct SqlField<'a, T> {
    pub alias: Option<&'a str>,
    pub table_alias: &'a str,
    pub field_name: &'static str,
    value_type: PhantomData<T>,
}

impl<'a, T> SqlField<'a, T> {
    pub fn new(table_alias: &'a str, field_name: &'static str) -> Self {
        Self {
            alias: None,
            table_alias: table_alias,
            field_name: field_name,
            value_type: PhantomData::default(),
        }
    }

    pub fn with_alias(self, alias: &'a str) -> Self {
        Self {
            alias: Some(alias),
            ..self
        }
    }

    pub fn with_table_alias(self, alias: &'a str) -> Self {
        Self {
            table_alias: alias,
            ..self
        }
    }

    pub fn twn(&self) -> Cow<'a, str> {
        format!("{}.{}", self.table_alias, self.field_name).into()
    }
}

impl<'a, T: 'a> SqlField<'a, T>
where
    T: Into<SqlValue<'a>>,
{
    pub fn v_eq<V: Into<T>>(&self, v: V) -> Wheres<'a> {
        Wheres::equal(self.twn(), v.into())
    }

    pub fn v_in<V: Into<T>>(&self, vs: Vec<V>) -> Wheres<'a> {
        Wheres::r#in(self.twn(), vs.into_iter().map(|v| v.into()).collect())
    }
}

impl<'a> SqlField<'a, Text> {
    pub fn v_ilike<V: AsRef<str>>(&self, v: V, exact: ILikeType) -> Wheres<'a> {
        Wheres::ilike(self.twn(), v.as_ref(), exact)
    }
}

impl<'a> SqlField<'a, i64> {
    pub fn v_gt<V: Into<i64>>(&self, v: V) -> Wheres<'a> {
        Wheres::compare(self.twn(), ">", v.into())
    }

    pub fn v_lt<V: Into<i64>>(&self, v: V) -> Wheres<'a> {
        Wheres::compare(self.twn(), "<", v.into())
    }

    pub fn v_ge<V: Into<i64>>(&self, v: V) -> Wheres<'a> {
        Wheres::compare(self.twn(), ">=", v.into())
    }

    pub fn v_le<V: Into<i64>>(&self, v: V) -> Wheres<'a> {
        Wheres::compare(self.twn(), "<=", v.into())
    }
}
