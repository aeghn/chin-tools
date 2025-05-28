#[derive(Clone, Copy)]
pub enum DbType {
    Sqlite,
    Postgres,
}

#[derive(Clone, Copy)]
pub enum LogicFieldType {
    Bool,
    I8,
    I16,
    I32,
    I64,
    F64,
    Varchar(u16),
    Text,
    Blob,
    Timestamptz,
    Timestamp,
}