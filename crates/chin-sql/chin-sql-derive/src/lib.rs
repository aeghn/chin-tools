use proc_macro::TokenStream;

mod tablecreatesql;
mod sqlinserter;

#[proc_macro_derive(GenerateTableSchema, attributes(gts_primary, gts_length, gts_type))]
pub fn generate_table_schema(input: TokenStream) -> TokenStream {
    tablecreatesql::table_schema(input)
}


#[proc_macro_derive(ChinSqlCrud)]
pub fn ChnotSqlExtra(input: TokenStream) -> TokenStream {
    sqlinserter::sql_inserter(input)
}