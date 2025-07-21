use chin_sql::DbType;
use chin_sql::str_type::Varchar;
use chin_sql_derive::GenerateTableSchema;
use chrono::DateTime;
use chrono::FixedOffset;

#[allow(dead_code)]
#[derive(GenerateTableSchema)]
struct ExampleTable {
    #[gts_primary]
    id: Varchar<211>,

    create_at: Option<DateTime<FixedOffset>>,

    #[gts_type = "bool"]
    #[gts_unique]
    create_at2: Option<DateTime<FixedOffset>>,
}

#[test]
fn table_generate() {
    println!("{:#?}", ExampleTable::create_sql().to_owned_sql().sqls(DbType::Postgres));

    assert_eq!("example_table", ExampleTable::TABLE);

    assert_eq!("create_at", ExampleTable::CREATE_AT);
}
