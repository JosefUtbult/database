use core::panic;
use std::collections::HashMap;

use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::Fields;
use syn::ItemStruct;

use crate::ParsedInput;
use crate::parse_input;
use crate::to_dromedar_case;

struct FieldInfo<'a> {
    field_type: syn::Type,
    parent: &'a ItemStruct,
    // abs_paths: Vec<String>,
}

pub type InheritencePath = Vec<String>;
pub type StructInhertice = HashMap<String, Vec<InheritencePath>>;

fn build_struct_names(structs: &Vec<ItemStruct>) -> Vec<String> {
    structs
        .clone()
        .iter()
        .map(|instance| instance.ident.to_string())
        .collect()
}

fn map_struct_inheritence_recursivly(
    focus: &ItemStruct,
    focus_inheritence: &InheritencePath,
    structs: &Vec<ItemStruct>,
    struct_names: &Vec<String>,
    res: &mut StructInhertice,
) {
    match &focus.fields {
        Fields::Named(fields_named) => {
            for field in fields_named.named.iter() {
                let field_ident = field.ident.as_ref().unwrap();
                let field_name = field_ident.to_string();
                let field_type = field.ty.clone();
                let field_type_string = &field_type.to_token_stream().to_string();

                // Ignore all non-struct members
                if !struct_names.contains(field_type_string) {
                    continue;
                }

                // Create a new path vector with the current path + the field name
                let mut current_path = focus_inheritence.clone();
                current_path.push(field_name);

                // Push this new path to the inheritance vector for the field type
                let inheritence_vector = res.entry(field_type_string.clone()).or_insert(Vec::new());
                inheritence_vector.push(current_path);
            }
        }
        _ => panic!("Only named fields are supported"),
    }
}

pub(crate) fn map_struct_inheritence(structs: &Vec<ItemStruct>) -> StructInhertice {
    let mut res = StructInhertice::new();
    let struct_names = build_struct_names(structs);

    for struct_instance in structs.iter() {
        let struct_instace_name = struct_instance.ident.to_token_stream().to_string();
        map_struct_inheritence_recursivly(
            struct_instance,
            &vec![struct_instace_name],
            structs,
            &struct_names,
            &mut res);
    }

    res
}

fn build_field_map(structs: &Vec<ItemStruct>) -> HashMap<String, FieldInfo<'_>> {
    let mut flattened_fields: HashMap<String, FieldInfo> = HashMap::new();
    let struct_names = build_struct_names(structs);

    for struct_instance in structs.iter() {
        eprintln!("Got struct {}", struct_instance.ident);

        match &struct_instance.fields {
            Fields::Named(fields_named) => {
                for field in fields_named.named.iter() {
                    let field_ident = field.ident.as_ref().unwrap();
                    let field_name = field_ident.to_string();
                    let field_type = field.ty.clone();

                    // Ignore all folder instances in the flattened fields
                    if struct_names.contains(&field_type.to_token_stream().to_string()) {
                        eprintln!("Got struct member {}", field_name);
                        continue;
                    }

                    if let Some(existing_field_info) = flattened_fields.get(&field_name) {
                        // Verify that the parent is the same
                        {
                            let existing_parent =
                                existing_field_info.parent.to_token_stream().to_string();
                            let current_parent = struct_instance.to_token_stream().to_string();

                            if existing_parent != current_parent {
                                panic!(
                                    "Got duplicate field name {} in multiple structs ({}, {})",
                                    field_name, existing_parent, current_parent
                                );
                            }
                        }

                        // Verify the type of the field
                        {
                            let existing_type =
                                existing_field_info.field_type.to_token_stream().to_string();
                            let current_type = field_type.to_token_stream().to_string();
                            if existing_type != current_type {
                                panic!(
                                    "Got multiple fields named {} with differing types ({}, {})",
                                    field_name, existing_type, current_type
                                );
                            }
                        }

                        // Append the current absolute path
                    } else {
                        let field = FieldInfo {
                            field_type,
                            parent: struct_instance,
                        };
                        flattened_fields.insert(field_name, field);
                    }
                }
            }
            _ => panic!("Only named fields are supported"),
        }
    }

    flattened_fields
}

pub(crate) fn build_flat_enums(parsed_input: &ParsedInput) -> TokenStream2 {
    let mut key_variants: Vec<TokenStream2> = Vec::new();
    let mut field_variants: Vec<TokenStream2> = Vec::new();

    let flattened_fields = build_field_map(&parsed_input.structs);
    for (field_name, field_info) in &flattened_fields {
        eprintln!(
            "Field name: {}, field type: {:?}",
            field_name,
            field_info.field_type.to_token_stream()
        );

        let field_name = format_ident!("{}", to_dromedar_case(&field_name));

        eprintln!("Field name: {}", field_name);

        let field_type = field_info.field_type.clone().to_token_stream();

        key_variants.push(quote! {#field_name});
        field_variants.push(quote! {#field_name(#field_type)});
    }

    let keys_enum_name = format_ident!("{}Key", parsed_input.name);
    let fields_enum_name = format_ident!("{}Field", parsed_input.name);

    quote! {
        pub enum #keys_enum_name {
            #(#key_variants),*
        }

        pub enum #fields_enum_name {
            #(#field_variants),*
        }
    }
}
