use super::{place_hoder::PlaceHolderType, sql_builder::SqlSeg, sql_value::SqlValue, wheres::Wheres};

pub struct SqlUpdater<'a> {
    table: &'a str,
    setters: Vec<(&'a str, SqlValue<'a>)>,
    wheres: Wheres<'a>,
}

impl<'a> SqlUpdater<'a> {
    pub fn new(table: &'a str) -> Self {
        SqlUpdater {
            table: &table,
            setters: vec![],
            wheres: Wheres::and([]),
        }
    }

    pub fn set_if_some<T: Into<SqlValue<'a>>>(mut self, key: &'a str, value: Option<T>) -> Self {
        if let Some(v) = value {
            self.setters.push((key, v.into()));
        }

        self
    }

    pub fn trans_if_some<T: Into<SqlValue<'a>>, F: FnOnce(V) -> T, V>(
        mut self,
        key: &'a str,
        value: Option<V>,
        trans: F,
    ) -> Self {
        if let Some(v) = value {
            self.setters.push((key, trans(v).into()));
        }

        self
    }

    pub fn set<T: Into<SqlValue<'a>>>(mut self, key: &'a str, v: T) -> Self {
        self.setters.push((key, v.into()));
        self
    }

    pub fn r#where(mut self, wheres: Wheres<'a>) -> Self {
        self.wheres = wheres;
        self
    }

    pub fn build(self, mut value_type: PlaceHolderType) -> Option<SqlSeg<'a>> {
        if self.setters.is_empty() {
            return None;
        }

        let mut sb = String::new();
        let mut values: Vec<SqlValue<'a>> = Vec::new();

        sb.push_str(" update ");
        sb.push_str(self.table);
        sb.push_str(" set ");

        let fields: Vec<String> = self
            .setters
            .into_iter()
            .map(|(key, v)| {
                values.push(v);
                format!(" {} = {} ", key, value_type.next())
            })
            .collect();
        sb.push_str(fields.join(", ").as_str());

        if let Some(filters) = self.wheres.build(&mut value_type) {
            sb.push_str(" where ");
            sb.push_str(filters.seg.as_str());

            values.extend(filters.values);
        }

        Some(SqlSeg { seg: sb, values })
    }
}
