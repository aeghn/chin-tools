mod fieldhandler;

use fieldhandler::field_stream;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Field, Fields, parse_macro_input};

pub(crate) fn table_schema(input: TokenStream) -> TokenStream {
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
    let mut constants = TokenStream2::default();
    for f in fields.into_iter() {
        let field_name = f.ident.as_ref().unwrap();
        let fname = format_ident!("{}", upcase(field_name.to_string().as_ref()));
        let field_name = field_name.to_string();
        constants.extend(quote! { pub const #fname: &'static str = #field_name; });
    }

    let Ok(create_sql) = table_create_struct_stream(&table_name, &fields.iter().collect()) else {
        return syn::Error::new(input.span(), "Unable to create CreateTableSql")
            .to_compile_error()
            .into();
    };

    let expanded = quote! {
        impl #name {
            pub const TABLE: &'static str = #table_name;
            #constants

            pub fn create_sql() -> &'static chin_sql::CreateTableSql {
                #create_sql
            }
        }
    };

    TokenStream::from(expanded)
}

fn table_create_struct_stream(
    table_name: &str,
    fields: &Vec<&Field>,
) -> Result<TokenStream2, syn::Error> {
    let mut column_structs = TokenStream2::default();
    let mut pkey = TokenStream2::default();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let column_name = field_name.to_string();
        let table_field = field_stream(column_name.as_str(), field)?;
        column_structs.extend(table_field);
        column_structs.extend(quote! {, });

        for attr in &field.attrs {
            if attr.path().is_ident("gts_primary") {
                pkey.extend(quote! { #column_name, });
            }
        }
    }

    Ok(quote! {
        &chin_sql::CreateTableSql {
            table_name: #table_name,
            fields: &[ #column_structs ],
            pkey: &[ #pkey ],
        }
    })
}

fn camel2snake(name: &str) -> String {
    let mut table_name = String::new();
    let chars: Vec<char> = name.to_string().chars().collect();
    for i in 0..chars.len() {
        if i == 0 {
            table_name.push(chars[i].to_ascii_lowercase());
            continue;
        }

        let cur = chars.get(i).unwrap();
        let last = chars.get(i - 1).unwrap().is_ascii_uppercase();
        let next = chars.get(i + 1).is_none_or(|c| c.is_ascii_uppercase());

        if cur.is_ascii_uppercase() && (!last || !next) {
            table_name.push('_');
        }
        table_name.push(cur.to_ascii_lowercase());
    }

    table_name
}

fn upcase(src: &str) -> String {
    src.chars().map(|e| e.to_ascii_uppercase()).collect()
}
