use proc_macro::TokenStream;

mod tablecreatesql;

#[proc_macro_derive(GenerateTableSql, attributes(gts_primary, gts_length, gts_type))]
pub fn generate_table_sql(input: TokenStream) -> TokenStream {
    tablecreatesql::generate_table_sql(input)
}
