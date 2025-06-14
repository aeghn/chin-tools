use proc_macro::TokenStream;

mod sql_crud;
mod tablecreatesql;

#[proc_macro_derive(GenerateTableSchema, attributes(gts_primary, gts_length, gts_type))]
pub fn generate_table_schema(input: TokenStream) -> TokenStream {
    tablecreatesql::table_schema(input)
}

#[proc_macro_derive(ChinSqlCrud)]
pub fn chin_sql_crud(input: TokenStream) -> TokenStream {
    let mut tokens = TokenStream::default();
    tokens.extend(sql_crud::crud_tools(input));

    tokens
}
