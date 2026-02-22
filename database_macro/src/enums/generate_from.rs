use core::panic;

use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};

use crate::{
    DataStructure,
    casing::to_camel_case,
    data_structure::{
        absolute_path::{AbsolutePath, AbsolutePathField},
        struct_data::{StructData, TypeNames},
    },
};

pub(crate) fn generate_abs_from(
    crate_path: &TokenStream2,
    _data_structure: &DataStructure,
    struct_data: &StructData,
) -> TokenStream2 {
    let abs_key_enum = struct_data.type_names.abs_key_enum.clone();
    let abs_field_enum = struct_data.type_names.abs_field_enum.clone();

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
            Self::#field_name(_) => #abs_key_enum::#field_name
        });
    }

    // Then, child structs
    for (_child_name, child_field_vector) in struct_data.field_to_child_struct_map.iter() {
        for field in child_field_vector.iter() {
            let field_name = format_ident!("{}", to_camel_case(field));

            abs_field_to_abs_key_match.push(quote! {
                Self::#field_name(field) => #abs_key_enum::#field_name(field.to_key())
            });
        }
    }

    quote! {
        impl #crate_path::ToKey<#abs_key_enum> for #abs_field_enum {
            fn to_key(&self) -> #abs_key_enum {
                match &self {
                    #(#abs_field_to_abs_key_match,)*
                }
            }
        }
    }
}

fn recurse_absolute_path(
    abs_path: &mut AbsolutePath,
    field_content: &TokenStream2,
) -> (TokenStream2, TokenStream2) {
    // Note: abs path is reversed
    if let Some(field) = abs_path.pop() {
        match field {
            AbsolutePathField::NonStruct(field_data) => {
                let field_name = format_ident!("{}", to_camel_case(&field_data.name));

                let key_stream = quote! {
                    #field_name
                };

                let field_stream = quote! {
                    #field_name(#field_content)
                };

                (key_stream, field_stream)
            }
            AbsolutePathField::Struct((field_name, struct_data)) => {
                let field_name = format_ident!("{}", to_camel_case(&field_name));
                let struct_key_enum = struct_data.type_names.abs_key_enum.clone();
                let struct_field_enum = struct_data.type_names.abs_key_enum.clone();

                let (child_keys, child_fields) = recurse_absolute_path(abs_path, field_content);

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

pub(crate) fn abs_path_to_token_stream(
    abs_path: &AbsolutePath,
    field_content: &TokenStream2,
    abs_key_enum: &Ident,
    abs_field_enum: &Ident,
) -> (TokenStream2, TokenStream2) {
    let mut abs_path = abs_path.clone();
    abs_path.reverse();
    let (child_keys, child_fields) = recurse_absolute_path(&mut abs_path, field_content);

    let abs_key_stream = quote! {
        #abs_key_enum::#child_keys
    };

    let abs_field_stream = quote! {
        #abs_field_enum::#child_fields
    };

    (abs_key_stream, abs_field_stream)
}

pub(crate) fn generate_flat_from(
    crate_path: &TokenStream2,
    data_structure: &DataStructure,
) -> TokenStream2 {
    let root_struct = data_structure.root_struct.as_ref().unwrap();

    let abs_key_enum = root_struct.type_names.abs_key_enum.clone();
    let flat_key_enum = root_struct.type_names.flat_key_enum.clone();
    let flat_field_enum = root_struct.type_names.flat_field_enum.clone();

    let mut flat_field_to_flat_key_match: Vec<TokenStream2> = Vec::new();
    let mut abs_key_to_flat_key_match: Vec<TokenStream2> = Vec::new();
    let mut abs_field_to_flat_field_match: Vec<TokenStream2> = Vec::new();

    for (flat_field, abs_paths) in root_struct.field_to_abs_path_map.iter() {
        let field_name = format_ident!("{}", to_camel_case(&flat_field));

        flat_field_to_flat_key_match.push(quote! {
            Self::#field_name(_) => #flat_key_enum::#field_name
        });

        for abs_path in abs_paths.iter() {
            let self_ident = format_ident!("Self");
            let value_stream = quote!(value);
            let (child_key, child_field) =
                abs_path_to_token_stream(abs_path, &value_stream, &abs_key_enum, &self_ident);

            abs_key_to_flat_key_match.push(quote! {
                #child_key => Self::#field_name
            });

            abs_field_to_flat_field_match.push(quote! {
                #child_field => #flat_field_enum::#field_name(value)
            });
        }
    }

    quote! {
        impl #crate_path::ToKey<#flat_key_enum> for #flat_field_enum {
            fn to_key(&self) -> #flat_key_enum {
                match self {
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
