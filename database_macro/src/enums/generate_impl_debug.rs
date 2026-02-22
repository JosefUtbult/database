use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::{DataStructure, casing::to_camel_case, data_structure::struct_data::StructData};

pub(crate) fn generate_impl_debug(
    _crate_path: &TokenStream2,
    _data_structure: &DataStructure,
    struct_data: &StructData,
) -> TokenStream2 {
    let abs_key_enum = struct_data.type_names.abs_key_enum.clone();
    let abs_field_enum = struct_data.type_names.abs_field_enum.clone();

    let mut key_match: Vec<TokenStream2> = Vec::new();
    let mut field_match: Vec<TokenStream2> = Vec::new();

    // Start with non-structs
    for field in struct_data.fields.iter() {
        // Ignore child struct fields
        if struct_data
            .field_to_child_struct_map
            .get(&field.ty_string)
            .is_some()
        {
            continue;
        }

        let field_name = format_ident!("{}", to_camel_case(&field.name));

        let key_string = format!("{}::{}", abs_key_enum.to_string(), field_name.to_string());
        key_match.push(quote! {
            Self::#field_name => write!(f, #key_string)
        });

        let field_string = format!("{}::{}", abs_field_enum.to_string(), field_name.to_string());
        field_match.push(quote! {
            Self::#field_name(_) => write!(f, #field_string)
        });
    }

    // Then, child structs
    for (_child_name, child_field_vector) in struct_data.field_to_child_struct_map.iter() {
        for field in child_field_vector.iter() {
            let field_name = format_ident!("{}", to_camel_case(field));

            let key_string = format!(
                "{}::{}({{:?}})",
                abs_key_enum.to_string(),
                field_name.to_string()
            );
            key_match.push(quote! {
                Self::#field_name(key) => write!(f, #key_string, key)
            });

            let field_string = format!(
                "{}::{}({{:?}})",
                abs_field_enum.to_string(),
                field_name.to_string()
            );

            field_match.push(quote! {
                Self::#field_name(field) => write!(f, #field_string, field)
            });
        }
    }

    quote! {
        impl core::fmt::Debug for #abs_key_enum {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#key_match,)*
                }
            }
        }

        impl core::fmt::Debug for #abs_field_enum {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#field_match,)*
                }
            }
        }
    }
}
