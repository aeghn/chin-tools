mod fieldhandler;


use chin_tools_base::DbType;
use fieldhandler::field_to_sql_type;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Field, Fields, parse_macro_input};

pub(crate) fn generate_table_sql(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new(
                    input.span(),
                    "GenerateTable can only be applied to structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new(input.span(), "GenerateTable can only be applied to structs")
                .to_compile_error()
                .into();
        }
    };

    let table_name = camel2snake(name.to_string().as_str());
    let fields = fields.iter().collect();
    let sqlite_sql = generate_sql(&fields, DbType::Sqlite, &table_name);
    let pg_sql = generate_sql(&fields, DbType::Postgres, &table_name);

    let mut func_stream = TokenStream2::default();
    for f in fields.into_iter() {
        let field_name = f.ident.as_ref().unwrap();
        let fname = format_ident!("{}", upcase(field_name.to_string().as_ref()));
        let field_name = field_name.to_string();
        func_stream.extend(quote! { pub const #fname: &'static str = #field_name; });
    }

    let expanded = quote! {
        impl #name {
            pub const TABLE: &'static str = #table_name;
            #func_stream

            pub fn schema(sql_type: chin_tools_base::DbType) -> &'static str {
                match sql_type {
                    DbType::Sqlite => #sqlite_sql,
                    DbType::Postgres => #pg_sql,
                }
            }

        }
    };

    TokenStream::from(expanded)
}

fn generate_sql(fields: &Vec<&Field>, db_type: DbType, table_name: &str) -> String {
    let mut columns = Vec::new();
    let mut primary_keys: Vec<String> = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let column_name = field_name.to_string();
        let column_type = field_to_sql_type(&field, db_type);

        for attr in &field.attrs {
            if attr.path().is_ident("gts_primary") {
                primary_keys.push(column_name.clone());
            }
        }

        let column_def = format!("{} {}", column_name, column_type.unwrap());

        columns.push(column_def);
    }

    if primary_keys.len() > 0 {
        let pk_columns = primary_keys.join(", ");
        columns.push(format!("PRIMARY KEY ({})", pk_columns));
    }

    let columns_str = columns.join(", ");

    format!(
        "CREATE TABLE IF NOT EXISTS {}({});",
        table_name, columns_str
    )
}

fn camel2snake(name: &str) -> String {
    let mut table_name = String::new();
    let mut last_down = false;
    let mut ll_down = false;
    name.to_string().chars().rev().for_each(|e| {
        if ll_down && !last_down {
            table_name.insert(0, '_');
        }
        table_name.insert(0, e.to_ascii_lowercase());
        
        ll_down = last_down;
        last_down = !e.is_uppercase();
    });

    table_name
}

fn upcase(src: &str) -> String {
    src.chars().map(|e| {e.to_ascii_uppercase()}).collect()
}