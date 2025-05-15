use chin_tools_types::DbType;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{Field, PathArguments, Type, TypePath};

pub(crate) fn field_to_sql_type(field: &Field, db_type: DbType) -> Result<String, syn::Error> {
    match &field.ty {
        Type::Path(type_path) => field_to_sql_type_path(field, db_type, type_path),
        Type::Group(group) => match group.elem.as_ref() {
            Type::Path(type_path) => field_to_sql_type_path(field, db_type, type_path),
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

fn field_to_sql_type_path(
    field: &Field,
    db_type: DbType,
    type_path: &TypePath,
) -> Result<String, syn::Error> {
    if let Some(segment) = type_path.path.segments.last() {
        let nullable = segment.ident.to_string().as_str() == "Option";
        let rt = find_attr_raw_rs_type(&field);
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
            "String" => parse_str(field, db_type)?,
            "SharedStr" => parse_str(field, db_type)?,
            "i32" => parse_i32(db_type)?,
            "i64" => parse_i64(db_type)?,
            "f32" => parse_f32(db_type)?,
            "f64" => parse_f64(db_type)?,
            "bool" => parse_bool(db_type)?,
            "DateTime<FixedOffset>" => parse_datetime_fixedoffset(db_type)?,
            "DateTime<Utc>" => parse_datetime_utc(db_type)?,
            _ => Err(syn::Error::new(
                field.span(),
                format!("Unkown Rust Type {:#?}", raw_rust_type.as_str()),
            ))?,
        };
        if nullable {
            Ok(sql_type)
        } else {
            Ok(sql_type + " not null")
        }
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
                    match &lit_int.lit {
                        syn::Lit::Int(lit_int) => return Some(lit_int.base10_parse()),
                        _ => {}
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

fn find_attr_raw_rs_type(field: &Field) -> Option<Result<String, syn::Error>> {
    let mut flag = false;
    for attr in &field.attrs {
        if attr.path().is_ident("gts_type") {
            flag = true;
            let meta = &attr.meta;
            if let syn::Meta::NameValue(name_value) = meta {
                if let syn::Expr::Lit(lit_int) = &name_value.value {
                    match &lit_int.lit {
                        syn::Lit::Str(lit_int) => {
                            return Some(Ok(lit_int.to_token_stream().to_string()));
                        }
                        _ => {}
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

fn parse_str(field: &Field, db_type: DbType) -> Result<String, syn::Error> {
    match db_type {
        DbType::Sqlite => Ok("TEXT".to_owned()),
        DbType::Postgres => {
            if let Some(length) = find_attr_length(&field) {
                match length {
                    Ok(length) => Ok(format!("VARCHAR({})", length)),
                    Err(err) => {
                        return Err(syn::Error::new(field.span(), err.to_string()));
                    }
                }
            } else {
                Ok("TEXT".to_owned())
            }
        }
    }
}

fn parse_i32(db_type: DbType) -> Result<String, syn::Error> {
    match db_type {
        DbType::Sqlite => Ok("INTEGER".to_owned()),
        DbType::Postgres => Ok("int4".to_owned()),
    }
}
fn parse_i64(db_type: DbType) -> Result<String, syn::Error> {
    match db_type {
        DbType::Sqlite => Ok("INTEGER".to_owned()),
        DbType::Postgres => Ok("int8".to_owned()),
    }
}
fn parse_f32(db_type: DbType) -> Result<String, syn::Error> {
    match db_type {
        DbType::Sqlite => Ok("REAL".to_owned()),
        DbType::Postgres => Ok("real".to_owned()),
    }
}
fn parse_f64(db_type: DbType) -> Result<String, syn::Error> {
    match db_type {
        DbType::Sqlite => Ok("REAL".to_owned()),
        DbType::Postgres => Ok("double precision".to_owned()),
    }
}
fn parse_bool(db_type: DbType) -> Result<String, syn::Error> {
    match db_type {
        DbType::Sqlite => Ok("NUMERIC".to_owned()),
        DbType::Postgres => Ok("BOOLEAN".to_owned()),
    }
}
fn parse_datetime_fixedoffset(db_type: DbType) -> Result<String, syn::Error> {
    match db_type {
        DbType::Sqlite => Ok("TEXT".to_owned()),
        DbType::Postgres => Ok("TIMESTAMPTZ".to_owned()),
    }
}
fn parse_datetime_utc(db_type: DbType) -> Result<String, syn::Error> {
    match db_type {
        DbType::Sqlite => Ok("TEXT".to_owned()),
        DbType::Postgres => Ok("TIMESTAMPTZ".to_owned()),
    }
}
