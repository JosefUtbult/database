use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::{DataStructure, casing::to_camel_case, data_structure::struct_data::StructData};

fn generate_variant_count(
    crate_path: &TokenStream2,
    _data_structure: &DataStructure,
    struct_data: &StructData,
) -> TokenStream2 {
    let flat_key_enum = struct_data.type_names.flat_key_enum.clone();
    let flat_field_enum = struct_data.type_names.flat_field_enum.clone();

    let flat_path_count_name = struct_data.type_names.flat_path_count_name.clone();

    let flat_path_count = struct_data.field_to_field_abs_path_map.len();

    quote! {
        pub const #flat_path_count_name: usize = #flat_path_count;

        #[automatically_derived]
        impl #crate_path::VariantCount for #flat_field_enum {
            const COUNT: usize = #flat_path_count_name;
        }

        #[automatically_derived]
        impl #crate_path::VariantCount for #flat_key_enum {
            const COUNT: usize = #flat_path_count_name;
        }
    }
}

fn generate_from_flat_key_to_usize(
    crate_path: &TokenStream2,
    _data_structure: &DataStructure,
    struct_data: &StructData,
) -> TokenStream2 {
    let flat_key_enum = struct_data.type_names.flat_key_enum.clone();

    let mut all_fields: Vec<String> = struct_data
        .field_to_field_abs_path_map
        .iter()
        .map(|(field, _)| field.clone())
        .collect();

    all_fields.sort();
    eprintln!("All fields {:?}", all_fields);

    let all_variants: (Vec<TokenStream2>, Vec<TokenStream2>) = all_fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let field_name = format_ident!("{}", to_camel_case(&field));
            let to_usize = quote! {
                Self::#field_name => #index
            };
            let from_usize = quote! {
                #index => Some(Self::#field_name)
            };
            (to_usize, from_usize)
        })
        .collect();

    let (all_to_usize_matches, all_from_usize_matches) = all_variants;

    quote! {
        #[automatically_derived]
        impl #crate_path::ToFromUsize for #flat_key_enum {
            fn to_usize(&self) -> usize {
                match self {
                    #(#all_to_usize_matches,)*
                }
            }

            fn try_from_usize(value: usize) -> Option<Self> {
                match value {
                    #(#all_from_usize_matches,)*
                    _ => None
                }
            }
        }
    }
}

fn generate_database_traits(
    crate_path: &TokenStream2,
    _data_structure: &DataStructure,
    struct_data: &StructData,
) -> TokenStream2 {
    let abs_key_enum = struct_data.type_names.abs_key_enum.clone();
    let abs_field_enum = struct_data.type_names.abs_field_enum.clone();
    let flat_key_enum = struct_data.type_names.flat_key_enum.clone();
    let flat_field_enum = struct_data.type_names.flat_field_enum.clone();

    quote! {
        #[automatically_derived]
        impl #crate_path::AbsKeyConstraints for #abs_key_enum {}
        #[automatically_derived]
        impl #crate_path::AbsFieldConstraints<#abs_key_enum> for #abs_field_enum {}
        #[automatically_derived]
        impl #crate_path::FlatKeyConstraints<#abs_key_enum> for #flat_key_enum {}
        #[automatically_derived]
        impl #crate_path::FlatFieldConstraints<#abs_field_enum, #flat_key_enum> for #flat_field_enum {}
    }
}

pub(crate) fn generate_database_constraints(
    crate_path: &TokenStream2,
    data_structure: &DataStructure,
) -> TokenStream2 {
    let mut res = TokenStream2::new();
    let root_struct = data_structure.root_struct.as_ref().unwrap();
    res.extend(generate_variant_count(
        crate_path,
        data_structure,
        root_struct,
    ));
    res.extend(generate_from_flat_key_to_usize(
        crate_path,
        data_structure,
        root_struct,
    ));
    res.extend(generate_database_traits(
        crate_path,
        data_structure,
        root_struct,
    ));
    res
}
