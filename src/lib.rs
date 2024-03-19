mod db;

#[allow(non_snake_case)]
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{Data, DeriveInput, Fields};

#[proc_macro_derive(Serialize)]
pub fn Serialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);

    let name = &ast.ident;
    let struct_name = name.to_string();

    let mut fields_code = Vec::new();
    if let Data::Struct(ref data) = ast.data {
        if let Fields::Named(ref fields) = data.fields {
            for field in fields.named.iter() {
                let field_name = field.ident.as_ref().unwrap();
                let field_type = &field.ty;

                if let Some(sql_type) = get_sql_type(field_type) {
                    fields_code.push(quote! {
                        map.insert(stringify!(#field_name).to_string(), #sql_type.to_string());
                    });
                } else {
                    panic!(
                        "Unsupported type '{}' in struct '{}'",
                        quote!(#field_type),
                        struct_name
                    );
                }
            }
        }
    }

    let gen = quote! {
        impl #name {
            fn serialize(&self) -> (String, HashMap<String, String>) {
                let mut map = HashMap::new();
                #(#fields_code)*
                (stringify!(#name).to_string(), map)
            }
        }
    };

    gen.into()
}

fn get_sql_type(ty: &syn::Type) -> Option<&'static str> {
    match ty {
        syn::Type::Path(type_path) => {
            let segment = type_path.path.segments.last().unwrap();
            let ident = &segment.ident;
            match ident.to_string().as_str() {
                "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" => Some("INTEGER"),
                "f32" | "f64" => Some("FLOAT"),
                "String" => Some("TEXT"),
                "bool" => Some("BOOLEAN"),
                _ => None,
            }
        }
        _ => None,
    }
}

#[proc_macro_derive(Deserialize)]
pub fn Deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let mut fields_code = Vec::new();
    if let Data::Struct(ref data) = ast.data {
        if let Fields::Named(ref fields) = data.fields {
            for field in fields.named.iter() {
                let field_name = field.ident.as_ref().unwrap();
                let field_type = &field.ty;

                let field_deserialize_code = match field_type.to_token_stream().to_string().as_str()
                {
                    "String" => quote! { #field_name: value.to_owned() },
                    "isize" => quote! { #field_name: value.parse().unwrap_or(0) },
                    "bool" => quote! { #field_name: value.parse().unwrap_or(false) },
                    _ => quote! { #field_name: value.parse().unwrap_or_default() },
                };

                fields_code.push(field_deserialize_code);
            }
        }
    }

    let gen = quote! {
        impl #name {
            fn deserialize(map: &HashMap<String, String>) -> Result<Self, String> {
                let mut deserialized_struct = Self::default();

                #(
                    if let Some(value) = map.get(stringify!(#fields_code)) {
                        deserialized_struct.#fields_code = match value.parse() {
                            Ok(parsed_value) => parsed_value,
                            Err(_) => return Err(format!("Failed to parse field '{}'", stringify!(#fields_code))),
                        };
                    }
                )*

                Ok(deserialized_struct)
            }
        }
    };

    gen.into()
}
