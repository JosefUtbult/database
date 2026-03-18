use core::panic;

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::{
    DataStructure,
    casing::to_camel_case,
    data_structure::{absolute_path::AbsolutePathField, struct_data::StructData},
};

pub(crate) fn generate_abs_enums(
    _crate_path: &TokenStream2,
    data_structure: &DataStructure,
    struct_data: &StructData,
) -> TokenStream2 {
    let abs_key_enum = struct_data.type_names.abs_key_enum.clone();
    let abs_field_enum = struct_data.type_names.abs_field_enum.clone();
    let abs_folder_enum = struct_data.type_names.abs_folder_enum.clone();

    let mut abs_key_variants: Vec<TokenStream2> = Vec::new();
    let mut abs_field_variants: Vec<TokenStream2> = Vec::new();
    let mut abs_folder_variants: Vec<TokenStream2> = Vec::new();

    for field in struct_data.fields.iter() {
        let field_name = format_ident!("{}", to_camel_case(&field.name));
        if let Some(child_struct) = data_structure.struct_map.get(&field.ty_string) {
            let child_struct_abs_key = format_ident!("{}", child_struct.type_names.abs_key_enum);
            let child_struct_abs_field =
                format_ident!("{}", child_struct.type_names.abs_field_enum);
            let child_struct_abs_folder =
                format_ident!("{}", child_struct.type_names.abs_folder_enum);

            abs_key_variants.push(quote! {
                #field_name(#child_struct_abs_key)
            });

            abs_field_variants.push(quote! {
                #field_name(#child_struct_abs_field)
            });

            abs_folder_variants.push(quote! {
                #field_name
            });

            if !child_struct.field_to_folder_map.is_empty() {
                let inner_field_name = format_ident!("In{}", to_camel_case(&field.name));
                abs_folder_variants.push(quote! {
                    #inner_field_name(#child_struct_abs_folder)
                });
            }
        } else {
            let field_type = format_ident!("{}", field.ty_string);

            abs_key_variants.push(quote! {
                #field_name
            });

            abs_field_variants.push(quote! {
                #field_name(#field_type)
            });
        }
    }

    let mut result = quote! {
        #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
        pub enum #abs_key_enum {
            #(#abs_key_variants,)*
        }

        #[derive(PartialEq, Eq, Clone, Copy)]
        pub enum #abs_field_enum {
            #(#abs_field_variants,)*
        }
    };

    if !abs_folder_variants.is_empty() {
        result.extend(quote! {
            #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
            pub enum #abs_folder_enum {
                #(#abs_folder_variants,)*
            }
        });
    }

    result
}

pub(crate) fn generate_flat_enums(
    _crate_path: &TokenStream2,
    data_structure: &DataStructure,
) -> TokenStream2 {
    let root_struct = data_structure.root_struct.as_ref().unwrap();

    let flat_key_enum = root_struct.type_names.flat_key_enum.clone();
    let flat_field_enum = root_struct.type_names.flat_field_enum.clone();
    let flat_folder_enum = root_struct.type_names.flat_folder_enum.clone();

    let mut flat_key_variants: Vec<TokenStream2> = Vec::new();
    let mut flat_field_variants: Vec<TokenStream2> = Vec::new();
    let mut flat_folder_variants: Vec<TokenStream2> = Vec::new();

    for (flat_field, abs_field_vector) in root_struct.field_to_field_abs_path_map.iter() {
        let field_name = format_ident!("{}", to_camel_case(&flat_field));
        let abs_path = abs_field_vector.first().unwrap();

        let last_path = abs_path.last().unwrap();
        let field_type = match last_path {
            AbsolutePathField::Struct(_) => panic!(),
            AbsolutePathField::NonStruct(field_data) => {
                format_ident!("{}", field_data.ty_string.clone())
            }
        };

        flat_key_variants.push(quote! {
            #field_name
        });

        flat_field_variants.push(quote! {
            #field_name(#field_type)
        });
    }

    for (folder_type, _abs_paths) in root_struct.field_to_folder_map.iter() {
        let folder_type_ident = format_ident!("{}", to_camel_case(&folder_type));
        flat_folder_variants.push(quote! {
            #folder_type_ident
        });
    }

    // for folder_type in root_struct.all_child_folders.iter() {
    //     let folder_type = format_ident!("{}", to_camel_case(&folder_type));
    //     flat_folder_variants.push(quote! {
    //         #folder_type
    //     });
    // }

    quote! {
        #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
        pub enum #flat_key_enum {
            #(#flat_key_variants,)*
        }

        #[derive(PartialEq, Eq, Clone, Copy)]
        pub enum #flat_field_enum {
            #(#flat_field_variants,)*
        }

        #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
        pub enum #flat_folder_enum {
            #(#flat_folder_variants,)*
        }
    }
}
