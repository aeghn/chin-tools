use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use syn::{Data, DeriveInput, Field, Fields, parse_macro_input, spanned::Spanned};

pub fn crud_tools(input: TokenStream) -> TokenStream {
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

    let fields: Vec<&Field> = fields.iter().collect();

    let tquery_by_pkey = query_by_pkey(&fields);
    let tto_sql_inserter = to_sql_inserter(&fields);
    let expanded = quote! {
        impl #name {
            #tquery_by_pkey
            #tto_sql_inserter
        }
    };

    TokenStream::from(expanded)
}

pub fn to_sql_inserter(fields: &Vec<&Field>) -> TokenStream2 {
    let mut func_stream = TokenStream2::default();
    for f in fields.iter() {
        let field_name = f.ident.as_ref().unwrap();
        let field_name = field_name.to_string();
        let db_field_ident = format_ident!("{}", field_name.to_uppercase());
        let field_indent = format_ident!("{}", field_name);
        func_stream.extend(quote! { .field(Self::#db_field_ident, self.#field_indent) });
    }

    quote! {
        pub fn to_sql_inserter(self) -> chin_sql::SqlInserter<'static> {
            chin_sql::SqlInserter::new(Self::TABLE)
                #func_stream
        }
    }
}

pub fn query_by_pkey(fields: &Vec<&Field>) -> TokenStream2 {
    let pkey_fields: Vec<&&Field> = fields
        .iter()
        .filter(|f| {
            for attr in &f.attrs {
                if attr.path().is_ident("gts_primary") {
                    return true;
                }
            }
            false
        })
        .collect();

    if pkey_fields.is_empty() {
        return TokenStream2::default();
    }
    let len = pkey_fields.len();
    let mut params = TokenStream2::default();
    let mut sql = TokenStream2::default();
    for (n, key) in pkey_fields.into_iter().enumerate() {
        let field_name = key.ident.as_ref().unwrap();
        let tty = key.ty.clone();
        params.extend(quote! { #field_name });
        params.extend(quote! { : });
        params.extend(quote! { #tty });
        if n < len - 1 {
            params.extend(quote! {, });
        }
        let column_name = field_name.to_string().to_uppercase();
        let column_name = format_ident!("{}", column_name);
        sql.extend(quote! { chin_sql::Wheres::equal(Self::#column_name, #field_name), });
    }

    let expanded = quote! {
        pub fn query_by_pkey_sql(#params) -> chin_sql::SqlBuilder<'static> {
            chin_sql::SqlBuilder::read_all(Self::TABLE)
            .r#where(chin_sql::Wheres::and([
                #sql
            ]))
        }
    };

    expanded
}
