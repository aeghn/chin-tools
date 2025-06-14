use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Field, PathArguments, Type, TypePath};

pub(crate) fn field_stream(column_name: &str, field: &Field) -> Result<TokenStream2, syn::Error> {
    match &field.ty {
        Type::Path(type_path) => parse_field(column_name, field, type_path),
        Type::Group(group) => match group.elem.as_ref() {
            Type::Path(type_path) => parse_field(column_name, field, type_path),
            v => Err(syn::Error::new(
                field.span(),
                format!(
                    "Error compling, not ty type in group {:#?}, true is {:#?}",
                    field.to_token_stream().to_string(),
                    v
                ),
            )),
        },
        v => Err(syn::Error::new(
            field.span(),
            format!(
                "Error compling, not ty type {:#?}, true is {:#?}",
                field.to_token_stream().to_string(),
                v
            ),
        )),
    }
}

fn parse_field(
    column_name: &str,
    field: &Field,
    type_path: &TypePath,
) -> Result<TokenStream2, syn::Error> {
    if let Some(segment) = type_path.path.segments.last() {
        let nullable = segment.ident.to_string().as_str() == "Option";
        let rt = find_attr_alias_type(field);
        let raw_rust_type;
        if let Some(Ok(rt)) = rt {
            raw_rust_type = rt;
        } else {
            raw_rust_type = if nullable {
                if let PathArguments::AngleBracketed(ab) = &segment.arguments {
                    ab.args.to_token_stream().to_string()
                } else {
                    return Err(syn::Error::new(
                        field.span(),
                        format!(
                            "This field is optional, but there is not Generic Type {:#?}",
                            field.to_token_stream().to_string()
                        ),
                    ));
                }
            } else {
                segment.to_token_stream().to_string()
            };
        }

        let raw_rust_type = raw_rust_type.replace(" ", "").replace("\"", "");

        let sql_type = match raw_rust_type.as_str() {
            "String" => parse_str(field)?,
            "i32" => quote! { chin_sql::LogicFieldType::I32 },
            "i64" => quote! { chin_sql::LogicFieldType::I64 },
            "f32" => quote! { chin_sql::LogicFieldType::F64 },
            "f64" => quote! { chin_sql::LogicFieldType::F64 },
            "bool" => quote! { chin_sql::LogicFieldType::Bool },
            "DateTime<FixedOffset>" => quote! { chin_sql::LogicFieldType::Timestamptz },
            "DateTime<Utc>" => quote! { chin_sql::LogicFieldType::Timestamp },
            _ => Err(syn::Error::new(
                field.span(),
                format!("Unkown Rust Type {:#?}", raw_rust_type.as_str()),
            ))?,
        };

        // This is the corrected `quote!` block
        let not_null = !nullable;
        Ok(quote! {
            chin_sql::CreateTableField {
                name: #column_name,
                kind: #sql_type,
                not_null: #not_null,
            }
        })
    } else {
        Err(syn::Error::new(
            field.span(),
            format!(
                "Error compling, cannot find the field ident {:#?}",
                field.to_token_stream().to_string()
            ),
        ))
    }
}

fn find_attr_length(field: &Field) -> Option<Result<i32, syn::Error>> {
    let mut flag = false;
    for attr in &field.attrs {
        if attr.path().is_ident("gts_length") {
            flag = true;
            let meta = &attr.meta;
            if let syn::Meta::NameValue(name_value) = meta {
                if let syn::Expr::Lit(lit_int) = &name_value.value {
                    if let syn::Lit::Int(lit_int) = &lit_int.lit {
                        return Some(lit_int.base10_parse());
                    }
                }
            }
        }
    }
    if flag {
        Some(Err(syn::Error::new(field.span(), "Unable parse length")))
    } else {
        None
    }
}

fn find_attr_alias_type(field: &Field) -> Option<Result<String, syn::Error>> {
    let mut flag = false;
    for attr in &field.attrs {
        if attr.path().is_ident("gts_type") {
            flag = true;
            let meta = &attr.meta;
            if let syn::Meta::NameValue(name_value) = meta {
                if let syn::Expr::Lit(lit_int) = &name_value.value {
                    if let syn::Lit::Str(lit_int) = &lit_int.lit {
                        return Some(Ok(lit_int.to_token_stream().to_string()));
                    }
                }
            }
        }
    }
    if flag {
        Some(Err(syn::Error::new(field.span(), "Unable parse length")))
    } else {
        None
    }
}

fn parse_str(field: &Field) -> Result<TokenStream2, syn::Error> {
    if let Some(length) = find_attr_length(field) {
        match length {
            Ok(length) => {
                let length = u16::try_from(length)
                    .map_err(|e| syn::Error::new(field.span(), e.to_string()))?;
                Ok(quote! { chin_sql::LogicFieldType::Varchar(#length) })
            }
            Err(err) => Err(syn::Error::new(field.span(), err.to_string())),
        }
    } else {
        Ok(quote! { chin_sql::LogicFieldType::Text })
    }
}
