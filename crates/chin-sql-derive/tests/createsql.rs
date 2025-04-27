use chin_sql_derive::GenerateTableSql;
use chin_tools_base::DbType;
use chrono::DateTime;
use chrono::FixedOffset;

#[derive(GenerateTableSql)]
struct ExampleTable {
    #[gts_primary]
    #[gts_length = 211]
    id: String,
    create_at: Option<DateTime<FixedOffset>>,
    #[gts_type = "bool"]
    create_at2: Option<DateTime<FixedOffset>>,
}

#[test]
fn table_generate() {
    assert_eq!(
        "CREATE TABLE IF NOT EXISTS example_table(id TEXT not null, create_at TEXT, create_at2 NUMERIC, PRIMARY KEY (id));",
        ExampleTable::table_creation_sql(chin_tools_base::DbType::Sqlite)
    );
    assert_eq!(
        "CREATE TABLE IF NOT EXISTS example_table(id VARCHAR(211) not null, create_at TIMESTAMPTZ, create_at2 BOOLEAN, PRIMARY KEY (id));",
        ExampleTable::table_creation_sql(chin_tools_base::DbType::Postgres)
    );

    assert_eq!("example_table", ExampleTable::table_name());

    assert_eq!("create_at", ExampleTable::field_create_at())
}
