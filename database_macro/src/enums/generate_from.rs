use core::panic;

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::{
    DataStructure,
    casing::to_camel_case,
    data_structure::{
        absolute_path::{AbsolutePath, AbsolutePathField},
        struct_data::StructData,
    },
};

pub(crate) fn generate_abs_from(
    _crate_path: &TokenStream2,
    _data_structure: &DataStructure,
    struct_data: &StructData,
) -> TokenStream2 {
    let abs_key_enum = struct_data.abs_key_enum.clone();
    let abs_field_enum = struct_data.abs_field_enum.clone();

    let mut abs_field_to_abs_key_match: Vec<TokenStream2> = Vec::new();

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

        abs_field_to_abs_key_match.push(quote! {
            #abs_field_enum::#field_name(_) => #abs_key_enum::#field_name
        });
    }

    // Then, child structs
    for (_child_name, child_field_vector) in struct_data.field_to_child_struct_map.iter() {
        for field in child_field_vector.iter() {
            let field_name = format_ident!("{}", to_camel_case(field));

            abs_field_to_abs_key_match.push(quote! {
                #abs_field_enum::#field_name(value) => #abs_key_enum::#field_name(value.into())
            });
        }
    }

    quote! {
        impl From<#abs_field_enum> for #abs_key_enum {
            fn from(value: #abs_field_enum) -> Self {
                match value {
                    #(#abs_field_to_abs_key_match,)*
                }
            }
        }
    }
}

fn recurse_absolute_path(abs_path: &mut AbsolutePath) -> (TokenStream2, TokenStream2) {
    // Note: abs path is reversed
    if let Some(field) = abs_path.pop() {
        match field {
            AbsolutePathField::NonStruct(field_data) => {
                let field_name = format_ident!("{}", to_camel_case(&field_data.name));

                let key_stream = quote! {
                    #field_name
                };

                let field_stream = quote! {
                    #field_name(value)
                };

                (key_stream, field_stream)
            }
            AbsolutePathField::Struct((field_name, struct_data)) => {
                let field_name = format_ident!("{}", to_camel_case(&field_name));
                let struct_key_enum = struct_data.abs_key_enum.clone();
                let struct_field_enum = struct_data.abs_key_enum.clone();

                let (child_keys, child_fields) = recurse_absolute_path(abs_path);

                let key_stream = quote! {
                    #field_name(#struct_key_enum::#child_keys)
                };

                let field_stream = quote! {
                    #field_name(#struct_field_enum::#child_fields)
                };

                (key_stream, field_stream)
            }
        }
    } else {
        panic!("Corrupted absolute path")
    }
}

pub(crate) fn generate_flat_from(
    _crate_path: &TokenStream2,
    data_structure: &DataStructure,
) -> TokenStream2 {
    let root_struct = data_structure.root_struct.as_ref().unwrap();

    let abs_key_enum = root_struct.abs_key_enum.clone();
    let abs_field_enum = root_struct.abs_field_enum.clone();
    let flat_key_enum = root_struct.flat_key_enum.clone();
    let flat_field_enum = root_struct.flat_field_enum.clone();

    let mut flat_field_to_flat_key_match: Vec<TokenStream2> = Vec::new();
    let mut abs_key_to_flat_key_match: Vec<TokenStream2> = Vec::new();
    let mut abs_field_to_flat_field_match: Vec<TokenStream2> = Vec::new();

    for (flat_field, abs_paths) in root_struct.field_to_abs_path_map.iter() {
        let field_name = format_ident!("{}", to_camel_case(&flat_field));

        flat_field_to_flat_key_match.push(quote! {
            #flat_field_enum::#field_name(_) => #flat_key_enum::#field_name
        });

        for abs_path in abs_paths.clone().iter_mut() {
            abs_path.reverse();
            let (child_keys, child_fields) = recurse_absolute_path(abs_path);

            abs_key_to_flat_key_match.push(quote! {
                #abs_key_enum::#child_keys => #flat_key_enum::#field_name
            });

            abs_field_to_flat_field_match.push(quote! {
                #abs_field_enum::#child_fields => #flat_field_enum::#field_name(value)
            });
        }
    }

    quote! {
        impl From<#flat_field_enum> for #flat_key_enum {
            fn from(value: #flat_field_enum) -> Self {
                match value {
                    #(#flat_field_to_flat_key_match,)*
                }
            }
        }

        impl From<#abs_key_enum> for #flat_key_enum {
            fn from(value: #abs_key_enum) -> Self {
                match value {
                    #(#abs_key_to_flat_key_match,)*
                }
            }
        }
    }
}
