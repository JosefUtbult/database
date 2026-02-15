use core::panic;
use std::collections::HashMap;
use std::result;

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

// pub type InheritencePath = Vec<String>;

fn build_struct_names(structs: &Vec<ItemStruct>) -> Vec<String> {
    structs
        .clone()
        .iter()
        .map(|instance| instance.ident.to_string())
        .collect()
}

// fn build_initial_map(
//     focus: &ItemStruct,
//     focus_inheritence: &InheritencePath,
//     struct_names: &Vec<String>,
//     inheritence_res: &mut StructInhertice,
// ) {
//     match &focus.fields {
//         Fields::Named(fields_named) => {
//             for field in fields_named.named.iter() {
//                 let field_ident = field.ident.as_ref().unwrap();
//                 let field_name = field_ident.to_string();
//                 let field_type = field.ty.clone();
//                 let field_type_string = &field_type.to_token_stream().to_string();

//                 // Ignore all non-struct members
//                 if !struct_names.contains(field_type_string) {
//                     continue;
//                 }

//                 // Create a new path vector with the current path + the field name
//                 let mut current_path = focus_inheritence.clone();
//                 current_path.push(field_name);

//                 // Push this new path to the inheritance vector for the field type
//                 let inheritence_vector = inheritence_res
//                     .entry(field_type_string.clone())
//                     .or_insert(Vec::new());
//                 inheritence_vector.push(current_path);
//             }
//         }
//         _ => panic!("Only named fields are supported"),
//     }
// }

// fn recurse_build(initial_map: &StructInhertice, focus_name: &String) -> Vec<InheritencePath> {
//     let mut own_parent_paths: Vec<InheritencePath> = Vec::new();
//     if let Some(focused_map) = initial_map.get(focus_name) {
//         for current_parent_path in focused_map.iter() {
//             if current_parent_path.len() >= 2 {
//                 let parent_name = &current_parent_path[current_parent_path.len() - 2];
//                 let parent_paths = recurse_build(initial_map, parent_name);

//                 if parent_paths.len() == 0 {
//                     let mut combined_path = Vec::new();
//                     combined_path.push(current_parent_path[current_parent_path.len() - 2].clone());
//                     combined_path.push(current_parent_path[current_parent_path.len() - 1].clone());
//                     own_parent_paths.push(combined_path);
//                 } else {
//                     for parent_path in parent_paths.iter() {
//                         let mut combined_path = parent_path.clone();
//                         combined_path
//                             .push(current_parent_path[current_parent_path.len() - 2].clone());
//                         combined_path
//                             .push(current_parent_path[current_parent_path.len() - 1].clone());
//                         own_parent_paths.push(combined_path);
//                     }
//                 }
//             } else {
//                 panic!("Corrupted inheritence: {:?}", current_parent_path);
//             }
//         }
//     }
//     own_parent_paths
// }

// pub(crate) fn map_struct_inheritence(structs: &Vec<ItemStruct>) -> StructInhertice {
//     let mut initial_map = StructInhertice::new();
//     let struct_names = build_struct_names(structs);

//     for struct_instance in structs.iter() {
//         let struct_instace_name = struct_instance.ident.to_token_stream().to_string();
//         build_initial_map(
//             struct_instance,
//             &vec![struct_instace_name],
//             &struct_names,
//             &mut initial_map,
//         );
//     }

//     let mut result_map = StructInhertice::new();
//     let keys = initial_map.keys();

//     for key in keys {
//         let paths = recurse_build(&initial_map, key);
//         result_map.insert(key.clone(), paths);
//     }

//     result_map
// }

pub type InheritencePath = Vec<String>;
pub type StructInhertice = HashMap<String, Vec<InheritencePath>>;
type StructMap<'a> = HashMap<String, &'a ItemStruct>;

fn map_recursivly<'a>(
    struct_map: &StructMap,
    struct_names: &Vec<String>,
    struct_name: &String,
) -> Vec<InheritencePath> {
    let mut inheritence_res: Vec<InheritencePath> = Vec::new();
    let struct_item = struct_map.get(struct_name).unwrap();

    match &struct_item.fields {
        Fields::Named(fields_named) => {
            for field in fields_named.named.iter() {
                let field_ident = field.ident.as_ref().unwrap();
                let field_name = field_ident.to_string();
                let field_type = field.ty.clone();
                let field_type_string = &field_type.to_token_stream().to_string();

                // Build a current path to this field
                let current_path = {
                    let mut current_path: Vec<String> = Vec::new();
                    current_path.push(struct_name.clone());
                    current_path.push(field_name);
                    current_path
                };

                if !struct_names.contains(field_type_string) {
                    inheritence_res.push(current_path);
                } else {
                    // Map all field instances from the fields inheritences
                    let field_inheritence =
                        map_recursivly(struct_map, struct_names, &field_type_string);

                    let full_field_inheritence: Vec<InheritencePath> = field_inheritence
                        .iter()
                        .map(|field_instance| {
                            let mut res = current_path.clone();
                            for item in field_instance {
                                res.push(item.clone());
                            }
                            res
                        })
                        .collect();

                    for item in full_field_inheritence {
                        inheritence_res.push(item);
                    }
                }
            }
        }
        _ => panic!("Only named fields are supported"),
    }

    inheritence_res
}

pub(crate) fn map_struct_inheritence(structs: &Vec<ItemStruct>) -> StructInhertice {
    let mut inheritence = StructInhertice::new();

    // Populate a struct map
    let mut struct_map = StructMap::new();
    for struct_item in structs.iter() {
        let name = struct_item.ident.to_token_stream().to_string();
        struct_map.insert(name, struct_item);
    }

    let struct_names = build_struct_names(structs);
    for struct_name in struct_names.iter() {
        let field_map = map_recursivly(&struct_map, &struct_names, &struct_name);
        inheritence.insert(struct_name.clone(), field_map);
    }

    inheritence
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
