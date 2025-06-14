use proc_macro::TokenStream;

mod tablecreatesql;

#[proc_macro_derive(GenerateTableSchema, attributes(gts_primary, gts_length, gts_type))]
pub fn generate_table_schema(input: TokenStream) -> TokenStream {
    tablecreatesql::table_schema(input)
}
