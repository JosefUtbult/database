use core::panic;
use std::collections::HashMap;
use std::result;

use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::extra;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::Fields;
use syn::ItemStruct;
use syn::Type;
use syn::TypePath;

use crate::ParsedInput;
use crate::to_dromedar_case;

pub(crate) struct FieldInfo<'a> {
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

fn get_flattened_key_name(struct_name: &Ident) -> Ident {
    format_ident!("{}Key", struct_name)
}

fn get_flattened_field_name(struct_name: &Ident) -> Ident {
    format_ident!("{}Fields", struct_name)
}

fn get_abs_key_name(struct_name: &String) -> Ident {
    format_ident!("{}AbsKey", struct_name)
}

fn get_abs_field_name(struct_name: &String) -> Ident {
    format_ident!("{}AbsField", struct_name)
}

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

pub(crate) fn map_struct_inheritence(structs: &Vec<ItemStruct>) -> (StructInhertice, String) {
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

    // Find the root by eliminating structs that are children
    let mut root_elements: Vec<(String, Vec<InheritencePath>)> = Vec::new();
    for (potential_root_struct_name, potential_root_struct_inheritence) in &inheritence {
        let mut found_parent = false;
        for (compared_struct_name, compared_struct_inheritence) in &inheritence {
            if compared_struct_name == potential_root_struct_name {
                continue;
            }

            for path in compared_struct_inheritence {
                if path.contains(potential_root_struct_name) {
                    found_parent = true;
                    break;
                }
            }
        }

        if !found_parent {
            root_elements.push((
                potential_root_struct_name.clone(),
                potential_root_struct_inheritence.clone(),
            ));
        }
    }

    if root_elements.len() == 0 {
        panic!("No root struct found")
    } else if root_elements.len() != 1 {
        panic!("Found multiple root structs")
    }

    (inheritence, root_elements[0].0.clone())
}

pub(crate) type FlattenedFieldMap<'a> = HashMap<String, FieldInfo<'a>>;

pub(crate) fn build_flattened_field_map<'a>(structs: &'a Vec<ItemStruct>) -> FlattenedFieldMap<'a> {
    let mut flattened_fields: HashMap<String, FieldInfo> = HashMap::new();
    let struct_names = build_struct_names(structs);

    for struct_instance in structs.iter() {
        match &struct_instance.fields {
            Fields::Named(fields_named) => {
                for field in fields_named.named.iter() {
                    let field_ident = field.ident.as_ref().unwrap();
                    let field_name = field_ident.to_string();
                    let field_type = field.ty.clone();

                    // Ignore all folder instances in the flattened fields
                    if struct_names.contains(&field_type.to_token_stream().to_string()) {
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

pub(crate) fn build_flat_enums(
    parsed_input: &ParsedInput,
    flattened_field_map: &FlattenedFieldMap,
) -> TokenStream2 {
    let mut key_variants: Vec<TokenStream2> = Vec::new();
    let mut field_variants: Vec<TokenStream2> = Vec::new();

    for (field_name, field_info) in flattened_field_map {
        let field_name = format_ident!("{}", to_dromedar_case(&field_name));
        let field_type = field_info.field_type.clone().to_token_stream();

        key_variants.push(quote! {#field_name});
        field_variants.push(quote! {#field_name(#field_type)});
    }

    let flat_keys_enum_name = get_flattened_key_name(&parsed_input.name);
    let flat_fields_enum_name = get_flattened_field_name(&parsed_input.name);

    quote! {
        pub enum #flat_keys_enum_name {
            #(#key_variants),*
        }

        pub enum #flat_fields_enum_name {
            #(#field_variants),*
        }
    }
}

fn filter_struct_inheritence_paths(
    paths: &Vec<InheritencePath>,
    struct_name: &String,
) -> Vec<InheritencePath> {
    let mut filtered_paths: HashMap<String, Vec<String>> = HashMap::new();
    for path in paths {
        for field in path {
            if field == struct_name {
                continue;
            }
            if filtered_paths.get(field).is_none() {
                filtered_paths.insert(field.clone(), path.clone());
            }
            break;
        }
    }

    eprintln!("Map: {:?}", filtered_paths);

    let mut struct_inheritence = Vec::new();
    for (_, path) in filtered_paths {
        struct_inheritence.push(path);
    }

    struct_inheritence
}

pub(crate) fn build_abs_enums(
    parsed_input: &ParsedInput,
    root_struct_name: &String,
    _flattened_field_map: &FlattenedFieldMap,
    inheritence_map: &StructInhertice,
) -> TokenStream2 {
    let struct_names = build_struct_names(&parsed_input.structs);

    let mut result = TokenStream2::new();

    // Go through each struct, ignoring the root struct
    for (struct_name, struct_inheritence) in inheritence_map {
        // The root struct should use the resulting name that was specified by the macro
        let resulting_struct_name = if struct_name == root_struct_name {
            parsed_input.name.to_string()
        } else {
            struct_name.clone()
        };

        // As the struct inheritance vector contains all inheritance, for each sub structs
        // elements, it needs to be filtered
        let struct_inheritence = filter_struct_inheritence_paths(struct_inheritence, struct_name);

        eprintln!("Paths: {:?}", struct_inheritence);

        struct PathInfo {
            path: Vec<Ident>,
            path_key_type: Option<Ident>,
            path_field_type: Option<Ident>,
        }

        // Collect all struct and non struct fields
        let mut struct_path_infos = Vec::new();

        // For each sub-path in the full path vector
        for path in struct_inheritence {
            // Build a new vector of paths, up until a potential struct
            let mut path_info = PathInfo {
                path: Vec::new(),
                path_key_type: None,
                path_field_type: None,
            };

            for field in path {
                if field == struct_name.clone() {
                    continue;
                }

                eprintln!("Checking field {}", field);
                if struct_names.contains(&field) {
                    eprintln!("Found struct {}", field);
                    let _ = path_info.path_key_type.insert(get_abs_key_name(&field));
                    let _ = path_info.path_field_type.insert(get_abs_field_name(&field));

                    let as_string: Vec<_> = path_info
                        .path
                        .iter()
                        .map(|ident| ident.to_string())
                        .collect();
                    eprintln!("Path: {:?}", as_string);
                    break;
                }

                path_info.path.push(format_ident!("{}", field));
            }

            assert!(path_info.path.len() > 0);

            // If no type was specified, it means that it isn't a struct.
            // In that case, we want a type for the field, but not the key
            if path_info.path_key_type.is_none() {
                assert!(path_info.path_field_type.is_none());
                let last = path_info.path.last().unwrap();

                // Find the original struct item
                let struct_item = parsed_input
                    .structs
                    .iter()
                    .find(|item| item.ident == struct_name)
                    .unwrap();

                // Find the corresponding field
                let field = struct_item
                    .fields
                    .iter()
                    .find(|field| {
                        if let Some(ident) = &field.ident {
                            ident == last
                        } else {
                            false
                        }
                    })
                    .unwrap();

                // Format the type as an ident
                if let Type::Path(TypePath { path, .. }) = field.ty.clone() {
                    if let Some(seg) = path.segments.last() {
                        let type_ident = format_ident!("{}", seg.ident);
                        let _ = path_info.path_field_type.insert(type_ident);
                    } else {
                        panic!()
                    }
                } else {
                    panic!()
                }
            }

            // Format the paths as camel case
            path_info.path = path_info
                .path
                .iter_mut()
                .map(|field| format_ident!("{}", to_dromedar_case(&field.to_string())))
                .collect();

            // Push it to the structs paths
            struct_path_infos.push(path_info);
        }

        // The formated key paths might have a type, if it is a struct
        let formatted_key_paths: Vec<_> = struct_path_infos
            .iter_mut()
            .map(|path_info| {
                let path = path_info.path.clone();
                if let Some(key_type) = &path_info.path_key_type {
                    let key_type = key_type.clone();
                    quote! {#(#path)::* (#key_type)}
                } else {
                    quote! {#(#path)::*}
                }
            })
            .collect();

        // The formated field paths always have a type
        let formatted_field_paths: Vec<_> = struct_path_infos
            .iter_mut()
            .map(|path_info| {
                let path = path_info.path.clone();
                let field_type = if let Some(field_type) = &path_info.path_field_type {
                    field_type.clone()
                } else {
                    panic!()
                };
                quote! {#(#path)::* (#field_type)}
            })
            .collect();

        let abs_key_name = get_abs_key_name(&resulting_struct_name);
        let abs_field_name = get_abs_field_name(&resulting_struct_name);

        result.extend(quote! {
            pub enum #abs_key_name {
                #(#formatted_key_paths,)*
            }

            pub enum #abs_field_name {
                #(#formatted_field_paths,)*
            }
        })
    }

    result
}

pub(crate) fn build_flat_to_abs_impl(
    parsed_input: &ParsedInput,
    root_struct_name: &String,
    flattened_field_map: &FlattenedFieldMap,
    inheritence_map: &StructInhertice,
) -> TokenStream2 {
    let struct_names = build_struct_names(&parsed_input.structs);

    let mut result = TokenStream2::new();
    let root_struct_inheritence = inheritence_map.get(root_struct_name).unwrap();
    for path in root_struct_inheritence {
        let head = String::new();
        let tail = String::new();

        for field in path {
            if field == root_struct_name {
                continue;
            }
            let enum_field_name = to_dromedar_case(field);
        }
    }

    result
}
