use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::{
    DataStructure,
    casing::{to_camel_case, to_upper_snake_case},
    data_structure::{self, absolute_path::AbsolutePathField, struct_data::StructData},
};

pub(crate) fn generate_abs_enums(
    _crate_path: &TokenStream2,
    data_structure: &DataStructure,
    struct_data: &StructData,
) -> TokenStream2 {
    let abs_key_enum = struct_data.abs_key_enum.clone();
    let abs_field_enum = struct_data.abs_field_enum.clone();

    let mut abs_key_variants: Vec<TokenStream2> = Vec::new();
    let mut abs_field_variants: Vec<TokenStream2> = Vec::new();

    for field in struct_data.fields.iter() {
        let field_name = format_ident!("{}", to_camel_case(&field.name));
        if let Some(child_struct) = data_structure.struct_map.get(&field.ty_string) {
            let child_struct_abs_key = format_ident!("{}", child_struct.abs_key_enum);
            let child_struct_abs_field = format_ident!("{}", child_struct.abs_field_enum);

            abs_key_variants.push(quote! {
                #field_name(#child_struct_abs_key)
            });

            abs_field_variants.push(quote! {
                #field_name(#child_struct_abs_field)
            });
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

    if cfg!(feature = "debug") {
        quote! {
            pub enum #abs_key_enum {
                #(#abs_key_variants,)*
            }

            pub enum #abs_field_enum {
                #(#abs_field_variants,)*
            }
        }
    }
    else {
        quote! {
            #[derive(Eq, Clone, Ord)]
            pub enum #abs_key_enum {
                #(#abs_key_variants,)*
            }

            #[derive(Eq, Clone)]
            pub enum #abs_field_enum {
                #(#abs_field_variants,)*
            }
        }
    }
}

pub(crate) fn generate_flat_enums(
    _crate_path: &TokenStream2,
    data_structure: &DataStructure,
) -> TokenStream2 {
    let flat_key_enum = data_structure.root_struct.as_ref().unwrap().flat_key_enum.clone();
    let flat_field_enum = data_structure.root_struct.as_ref().unwrap().flat_field_enum.clone();

    let mut flat_key_variants: Vec<TokenStream2> = Vec::new();
    let mut flat_field_variants: Vec<TokenStream2> = Vec::new();

    for (flat_field, abs_field_vector) in data_structure
        .root_struct
        .as_ref()
        .unwrap()
        .field_to_abs_path_map
        .iter()
    {
        let field_name = format_ident!("{}", to_camel_case(&flat_field));
        let abs_path = abs_field_vector.first().unwrap();

        let last_path = abs_path.last().unwrap();
        let field_type = match last_path {
            AbsolutePathField::Struct(_) => panic!(),
            AbsolutePathField::NonStruct(field_data) => format_ident!("{}", field_data.ty_string.clone()),
        };

        flat_key_variants.push(quote! {
            #field_name
        });

        flat_field_variants.push(quote! {
            #field_name(#field_type)
        });
    }

    if cfg!(feature = "debug") {
        quote! {
            pub enum #flat_key_enum {
                #(#flat_key_variants,)*
            }

            pub enum #flat_field_enum {
                #(#flat_field_variants,)*
            }
        }
    } else {
        quote! {
            #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
            pub enum #flat_key_enum {
                #(#flat_key_variants,)*
            }

            #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
            pub enum #flat_field_enum {
                #(#flat_field_variants,)*
            }
        }
    }
}
