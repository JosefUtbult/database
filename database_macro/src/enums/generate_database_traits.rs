use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::{
    DataStructure,
    casing::{to_camel_case, to_upper_snake_case},
    data_structure::absolute_path::compare_abs_paths,
    enums::abs_path_to_token_stream,
};

fn generate_variant_count(
    crate_path: &TokenStream2,
    data_structure: &DataStructure,
) -> TokenStream2 {
    let root_struct = data_structure.root_struct.as_ref().unwrap();

    let abs_key_enum = root_struct.type_names.abs_key_enum.clone();
    let abs_field_enum = root_struct.type_names.abs_field_enum.clone();
    let abs_folder_enum = root_struct.type_names.abs_folder_enum.clone();

    let flat_key_enum = root_struct.type_names.flat_key_enum.clone();
    let flat_field_enum = root_struct.type_names.flat_field_enum.clone();
    let flat_folder_enum = root_struct.type_names.flat_folder_enum.clone();

    let abs_path_count_name = root_struct.type_names.abs_path_count_name.clone();
    let flat_path_count_name = root_struct.type_names.flat_path_count_name.clone();

    let abs_path_count = root_struct.abs_path_count;
    let flat_path_count = root_struct.field_to_abs_path_map.len();

    quote! {
        pub const #abs_path_count_name: usize = #abs_path_count;
        pub const #flat_path_count_name: usize = #flat_path_count;

        #[automatically_derived]
        impl #crate_path::VariantCount for #abs_key_enum {
            const COUNT: usize = #abs_path_count_name;
        }

        #[automatically_derived]
        impl #crate_path::VariantCount for #abs_field_enum {
            const COUNT: usize = #abs_path_count_name;
        }

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

fn generate_all_variants(
    crate_path: &TokenStream2,
    data_structure: &DataStructure,
) -> TokenStream2 {
    let root_struct = data_structure.root_struct.as_ref().unwrap();

    let abs_key_enum = root_struct.type_names.abs_key_enum.clone();
    let abs_field_enum = root_struct.type_names.abs_field_enum.clone();
    let abs_path_count = root_struct.abs_path_count;
    let abs_key_list_name = format_ident!(
        "{}_ALL_KEYS",
        to_upper_snake_case(&abs_key_enum.to_string())
    );

    let mut all_abs_paths = Vec::new();
    for (_, abs_path_vector) in root_struct.field_to_abs_path_map.iter() {
        for path in abs_path_vector.iter() {
            all_abs_paths.push(path.clone());
        }
    }

    all_abs_paths.sort_by(|lhs, rhs| compare_abs_paths(&lhs, &rhs));

    let all_keys: Vec<TokenStream2> = all_abs_paths
        .iter()
        .map(|abs_path| {
            abs_path_to_token_stream(&abs_path, &None, &abs_key_enum, &abs_field_enum).0
        })
        .collect();

    quote! {
        const #abs_key_list_name: [#abs_key_enum; #abs_path_count] = [
            #(#all_keys,)*
        ];

        #[automatically_derived]
        impl #crate_path::AllVariants for #abs_key_enum {
            const ALL_VARIANTS: &[Self] = &#abs_key_list_name;
        }
    }
}

fn generate_from_flat_key_to_usize(
    crate_path: &TokenStream2,
    data_structure: &DataStructure,
) -> TokenStream2 {
    let root_struct = data_structure.root_struct.as_ref().unwrap();

    let flat_key_enum = root_struct.type_names.flat_key_enum.clone();

    let mut all_fields: Vec<String> = root_struct
        .field_to_abs_path_map
        .iter()
        .map(|(field, _)| field.clone())
        .collect();

    all_fields.sort();

    let all_keys: Vec<TokenStream2> = all_fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let field_name = format_ident!("{}", to_camel_case(&field));
            quote! {
                #flat_key_enum::#field_name => #index
            }
        })
        .collect();

    quote! {
        #[automatically_derived]
        impl From<#flat_key_enum> for usize {
            fn from(value: #flat_key_enum) -> Self {
                match value {
                    #(#all_keys,)*
                }
            }
        }

        #[automatically_derived]
        impl #crate_path::UsizeConstraints<#flat_key_enum> for usize {}
    }
}

fn generate_database_traits(
    crate_path: &TokenStream2,
    data_structure: &DataStructure,
) -> TokenStream2 {
    let root_struct = data_structure.root_struct.as_ref().unwrap();

    let abs_key_enum = root_struct.type_names.abs_key_enum.clone();
    let abs_field_enum = root_struct.type_names.abs_field_enum.clone();
    let abs_folder_enum = root_struct.type_names.abs_folder_enum.clone();
    let flat_key_enum = root_struct.type_names.flat_key_enum.clone();
    let flat_field_enum = root_struct.type_names.flat_field_enum.clone();
    let flat_folder_enum = root_struct.type_names.flat_folder_enum.clone();

    quote! {
        #[automatically_derived]
        impl #crate_path::AbsKeyConstraints for #abs_key_enum {}
        #[automatically_derived]
        impl #crate_path::AbsFieldConstraints<#abs_key_enum> for #abs_field_enum {}
        #[automatically_derived]
        impl #crate_path::AbsFolderConstraints for #abs_folder_enum {}
        #[automatically_derived]
        impl #crate_path::FlatKeyConstraints<#abs_key_enum> for #flat_key_enum {}
        #[automatically_derived]
        impl #crate_path::FlatFieldConstraints<#abs_field_enum, #flat_key_enum> for #flat_field_enum {}
        #[automatically_derived]
        impl #crate_path::FlatFolderConstraints<#abs_folder_enum> for #flat_folder_enum {}
    }
}

pub(crate) fn generate_database_constraints(
    crate_path: &TokenStream2,
    data_structure: &DataStructure,
) -> TokenStream2 {
    let mut res = TokenStream2::new();
    res.extend(generate_variant_count(crate_path, data_structure));
    res.extend(generate_all_variants(crate_path, data_structure));
    res.extend(generate_from_flat_key_to_usize(crate_path, data_structure));
    res.extend(generate_database_traits(crate_path, data_structure));
    res
}
