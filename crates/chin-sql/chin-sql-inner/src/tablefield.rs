use std::marker::PhantomData;

use chin_tools_types::SharedStr;

use crate::{ILikeType, SqlValue, Wheres, str_type::Text};

pub struct SqlField<'a, T> {
    pub alias: Option<&'a str>,
    pub table_alias: Option<&'a str>,
    pub field_name: &'a str,
    value_type: PhantomData<T>,
}

impl<'a, T> SqlField<'a, T> {
    pub fn new(field_name: &'a str) -> Self {
        Self {
            alias: None,
            table_alias: None,
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
            table_alias: Some(alias),
            ..self
        }
    }

    pub fn with_opt_table_alias(self, alias: Option<&'a str>) -> Self {
        Self {
            table_alias: alias,
            ..self
        }
    }

    pub fn twn(&self) -> String {
        match self.table_alias {
            Some(alias) => {
                format!("{}.{}", alias, self.field_name)
            },
            None => {
                format!("{}", self.field_name)
            },
        }
    }

    fn identifier(&self) -> &str {
        /* if let Some(ta) = self.table_alias {
            format!("{}.{}", ta, self.field_name)
        } else {
            self.field_name.to_owned()
        } */
        todo!()
    }
}

impl<'a, T> SqlField<'a, T>
where
    T: Into<SqlValue<'a>>,
{
    pub fn v_eq<V: Into<T>>(&'a self, v: V) -> Wheres<'a> {
        Wheres::equal(&self.identifier(), v.into())
    }

    pub fn v_in<V: Into<T>>(&'a self, vs: Vec<V>) -> Wheres<'a> {
        Wheres::r#in(
            &self.identifier(),
            vs.into_iter().map(|v| v.into()).collect(),
        )
    }
}

impl<'a> SqlField<'a, Text> {
    pub fn v_ilike<V: AsRef<str>>(&'a self, v: V, exact: ILikeType) -> Wheres<'a> {
        Wheres::ilike(&self.identifier(), v.as_ref(), exact)
    }
}

impl<'a> SqlField<'a, i64> {
    pub fn v_gt<V: Into<i64>>(&'a self, v: V) -> Wheres<'a> {
        Wheres::compare(&self.identifier(), ">", v.into())
    }

    pub fn v_lt<V: Into<i64>>(&'a self, v: V) -> Wheres<'a> {
        Wheres::compare(&self.identifier(), "<", v.into())
    }

    pub fn v_ge<V: Into<i64>>(&'a self, v: V) -> Wheres<'a> {
        Wheres::compare(&self.identifier(), ">=", v.into())
    }

    pub fn v_le<V: Into<i64>>(&'a self, v: V) -> Wheres<'a> {
        Wheres::compare(&self.identifier(), "<=", v.into())
    }
}
