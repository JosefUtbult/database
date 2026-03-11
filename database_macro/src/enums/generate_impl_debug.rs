use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};

use crate::{DataStructure, casing::to_camel_case, data_structure::struct_data::StructData};

fn build_fmt(enum_name: &Ident, match_vector: &Vec<TokenStream2>) -> TokenStream2 {
    if match_vector.len() > 0 {
        quote! {
            #[automatically_derived]
            impl core::fmt::Debug for #enum_name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(#match_vector,)*
                    }
                }
            }
        }
    } else {
        let stringified = enum_name.to_string();
        quote! {
            #[automatically_derived]
            impl core::fmt::Debug for #enum_name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, #stringified)
                }
            }
        }
    }
}

pub(crate) fn generate_abs_impl_debug(
    _crate_path: &TokenStream2,
    data_structure: &DataStructure,
    struct_data: &StructData,
) -> TokenStream2 {
    let abs_key_enum = struct_data.type_names.abs_key_enum.clone();
    let abs_field_enum = struct_data.type_names.abs_field_enum.clone();
    let abs_folder_enum = struct_data.type_names.abs_folder_enum.clone();

    let mut key_match: Vec<TokenStream2> = Vec::new();
    let mut field_match: Vec<TokenStream2> = Vec::new();
    let mut folder_match: Vec<TokenStream2> = Vec::new();

    // Start with non-structs
    for field in struct_data.fields.iter() {
        // Ignore child struct fields
        if struct_data
            .field_to_folder_map
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

        if struct_data.has_debug_derive {
            let field_string = format!(
                "{}::{}({{:?}})",
                abs_field_enum.to_string(),
                field_name.to_string()
            );

            field_match.push(quote! {
                Self::#field_name(value) => write!(f, #field_string, value)
            });
        } else {
            let field_string =
                format!("{}::{}", abs_field_enum.to_string(), field_name.to_string());

            field_match.push(quote! {
                Self::#field_name(_) => write!(f, #field_string)
            });
        }
    }

    // Then structs
    for field in struct_data.fields.iter() {
        // Ignore non child struct fields
        if !struct_data
            .field_to_folder_map
            .get(&field.ty_string)
            .is_some()
        {
            continue;
        }

        let folder_name = format_ident!("{}", to_camel_case(&field.name));
        let in_folder_name = format_ident!("In{}", to_camel_case(&field.name));

        let child_struct_data = data_structure.struct_map.get(&field.ty_string).unwrap();
        let child_abs_folder_enum = &child_struct_data.type_names.abs_folder_enum;

        let key_string = format!(
            "{}::{}({{:?}})",
            abs_key_enum.to_string(),
            folder_name.to_string()
        );
        key_match.push(quote! {
            Self::#folder_name(key) => write!(f, #key_string, key)
        });

        let field_string = format!(
            "{}::{}({{:?}})",
            abs_field_enum.to_string(),
            folder_name.to_string()
        );

        field_match.push(quote! {
            Self::#folder_name(field) => write!(f, #field_string, field)
        });

        let folder_string = format!(
            "{}::{}",
            child_abs_folder_enum.to_string(),
            folder_name.to_string()
        );

        folder_match.push(quote! {
            Self::#folder_name => write!(f, #folder_string)
        });

        if !data_structure
            .struct_map
            .get(&field.ty_string)
            .unwrap()
            .field_to_folder_map
            .is_empty()
        {
            let in_folder_string = format!(
                "{}::{}({{:?}})",
                child_abs_folder_enum.to_string(),
                folder_name.to_string()
            );
            folder_match.push(quote! {
                Self::#in_folder_name(folder) => write!(f, #in_folder_string, folder)
            });
        }
    }

    let key_stream = build_fmt(&abs_key_enum, &key_match);
    let field_stream = build_fmt(&abs_field_enum, &field_match);

    let mut result = quote! {
        #key_stream
        #field_stream
    };

    if !folder_match.is_empty() {
        let folder_stream = build_fmt(&abs_folder_enum, &folder_match);
        result.extend(folder_stream);
    }

    result
}

pub(crate) fn generate_flat_impl_debug(
    _crate_path: &TokenStream2,
    data_structure: &DataStructure,
) -> TokenStream2 {
    let root_struct = data_structure.root_struct.as_ref().unwrap();
    let flat_key_enum = root_struct.type_names.flat_key_enum.clone();
    let flat_field_enum = root_struct.type_names.flat_field_enum.clone();
    let flat_folder_enum = root_struct.type_names.flat_folder_enum.clone();

    let mut key_match: Vec<TokenStream2> = Vec::new();
    let mut field_match: Vec<TokenStream2> = Vec::new();

    for (field, _) in root_struct.field_to_field_abs_path_map.iter() {
        let field_name = format_ident!("{}", to_camel_case(&field));

        let key_string = format!("{}::{}", flat_key_enum.to_string(), field_name.to_string());
        key_match.push(quote! {
            Self::#field_name => write!(f, #key_string)
        });

        if data_structure.all_structs_has_debug_derive {
            let field_string = format!(
                "{}::{}({{:?}})",
                flat_field_enum.to_string(),
                field_name.to_string()
            );

            field_match.push(quote! {
                Self::#field_name(value) => write!(f, #field_string, value)
            });
        } else {
            let field_string = format!(
                "{}::{}",
                flat_field_enum.to_string(),
                field_name.to_string()
            );

            field_match.push(quote! {
                Self::#field_name(_) => write!(f, #field_string)
            });
        }
    }

    let folder_match: Vec<TokenStream2> = root_struct
        .field_to_folder_map
        .iter()
        .map(|(folder_type, _)| {
            let folder_type = format_ident!("{}", to_camel_case(folder_type));
            let folder_string = format!(
                "{}::{}",
                flat_folder_enum.to_string(),
                folder_type.to_string()
            );
            quote! {
                Self::#folder_type => write!(f, #folder_string)
            }
        })
        .collect();

    let key_stream = build_fmt(&flat_key_enum, &key_match);
    let field_stream = build_fmt(&flat_field_enum, &field_match);
    let folder_stream = build_fmt(&flat_folder_enum, &folder_match);

    quote! {
        #key_stream
        #field_stream
        #folder_stream
    }
}
