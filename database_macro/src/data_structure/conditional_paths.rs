use proc_macro2::Ident;
use quote::format_ident;
use std::collections::HashMap;

use crate::{
    DataStructure,
    casing::{to_camel_case, to_snake_case},
    data_structure::struct_data::{StructData, StructMap},
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Reference {
    parent_name: String,
    struct_name: String,
    parent_field: String,
}

type StructToReferenceMap = HashMap<String, Vec<Reference>>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct MultiPath {
    parent_name: String,
    struct_name: String,
    parent_fields: Vec<String>,
    sub_paths: MultiPathVector,
}

type MultiPathVector = Vec<Box<MultiPath>>;
type MultiPathMap = HashMap<String, MultiPathVector>;

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct FolderFocusPath {
    pub(crate) struct_name: Ident,
    pub(crate) parent_fields: Vec<Ident>,
    pub(crate) sub_paths: FolderFocusPathVector,
}

pub(crate) type FolderFocusPathVector = Vec<Box<FolderFocusPath>>;
pub(crate) type FolderFocusPathMap = HashMap<String, FolderFocusPathVector>;

#[derive(Debug)]
pub(crate) struct FieldInfo {
    pub(crate) variant_name: Ident,
    pub(crate) field_name: Ident,
    #[allow(dead_code)]
    pub(crate) value_struct_name: Ident,
}

#[derive(Debug)]
pub(crate) struct FolderInfo {
    pub(crate) folder_enum_name: Ident,
    pub(crate) folder_focus_variable: Ident,
    #[allow(dead_code)]
    pub(crate) abs_folder_key_enum: Ident,
    pub(crate) fields: Vec<FieldInfo>,
}

pub(crate) type AllFolderFields = HashMap<String, FolderInfo>;

fn recurse_reference_map(
    struct_data: &StructData,
    struct_map: &StructMap,
    struct_to_reference_map: &mut StructToReferenceMap,
) {
    // For each field that is a struct
    for child_struct_field in struct_data.fields.iter() {
        if struct_map.get(&child_struct_field.ty_string).is_none() {
            continue;
        }

        // Build a new reference and push it to the struct to reference map, if it doesn't already
        // exist
        let reference = Reference {
            parent_name: struct_data.name.clone(),
            struct_name: child_struct_field.ty_string.clone(),
            parent_field: child_struct_field.name.clone(),
        };

        let child_reference_vector = struct_to_reference_map
            .entry(child_struct_field.ty_string.clone())
            .or_insert(Vec::new());

        if !child_reference_vector.contains(&reference) {
            child_reference_vector.push(reference);
        }

        // Recurs over the child struct
        let child_struct_data = struct_map.get(&child_struct_field.ty_string).unwrap();
        recurse_reference_map(child_struct_data, struct_map, struct_to_reference_map);
    }
}

fn build_struct_to_reference_map(data_structure: &DataStructure) -> StructToReferenceMap {
    let mut struct_to_reference_map = StructToReferenceMap::new();

    let struct_data_map = &data_structure.struct_map;
    for (_, struct_data) in struct_data_map.iter() {
        recurse_reference_map(struct_data, &struct_data_map, &mut struct_to_reference_map);
    }

    struct_to_reference_map
}

fn build_initial_multi_path_vector(reference_vector: Vec<Reference>) -> MultiPathVector {
    let mut multi_path_vector = Vec::new();

    for reference in reference_vector {
        // Check if there already exists a multi path for the current references parent struct
        let existing_multi_path =
            multi_path_vector
                .iter_mut()
                .find_map(|multi_path: &mut Box<MultiPath>| {
                    if multi_path.parent_name == reference.parent_name {
                        Some(multi_path)
                    } else {
                        None
                    }
                });

        // If it exists, add the new field to the existing paths fields
        if let Some(existing_multi_path) = existing_multi_path {
            if !existing_multi_path
                .parent_fields
                .contains(&reference.parent_field)
            {
                existing_multi_path
                    .parent_fields
                    .push(reference.parent_field.clone());
            }
        }
        // Otherwise, create a new multi path
        else {
            let multi_path = MultiPath {
                parent_name: reference.parent_name,
                struct_name: reference.struct_name,
                parent_fields: vec![reference.parent_field],
                sub_paths: Vec::new(),
            };
            multi_path_vector.push(Box::new(multi_path));
        }
    }

    multi_path_vector
}

fn build_struct_to_multi_path_map(struct_to_reference_map: StructToReferenceMap) -> MultiPathMap {
    struct_to_reference_map
        .into_iter()
        .map(|(struct_name, reference_vector)| {
            let multi_path_vector = build_initial_multi_path_vector(reference_vector);
            (struct_name, multi_path_vector)
        })
        .collect()
}

fn recurse_expand_multi_path(
    current_struct_name: &String,
    root_struct_name: &String,
    multi_path_map: &mut MultiPathMap,
) -> MultiPathVector {
    // Retrieve the current multi map and filter out a list of indices to multi_paths that needs
    // expansion, and their names
    let needs_expansion: Vec<(usize, String)> = {
        let multi_path_vector = multi_path_map.get(current_struct_name).unwrap();

        multi_path_vector
            .iter()
            .enumerate()
            .filter(|(_, multi_path)| {
                // The root struct is the base case of the recursion
                if multi_path.parent_name == *root_struct_name {
                    false
                }
                // If the sub path of the multipath already is populated, that
                // means that some other step in the recursion already populated it
                else {
                    multi_path.sub_paths.is_empty()
                }
            })
            .map(|(index, multi_path)| (index, multi_path.parent_name.clone()))
            .collect()
    };

    let mut expanded_multi_paths: Vec<(usize, MultiPathVector)> = Vec::new();

    // Recurs over each child struct that needs expansion
    for (index, parent_name) in needs_expansion.iter() {
        let expanded = recurse_expand_multi_path(parent_name, root_struct_name, multi_path_map);
        expanded_multi_paths.push((*index, expanded));
    }

    // For each expanded path, insert it into the appropriate sub_path vector
    {
        let multi_path_vector = multi_path_map.get_mut(current_struct_name).unwrap();
        for (index, expanded_path_vector) in expanded_multi_paths {
            for multi_path in expanded_path_vector {
                multi_path_vector[index].sub_paths.push(multi_path);
            }
        }
    }

    multi_path_map.get(current_struct_name).unwrap().clone()
}

fn expand_multi_path(root_struct_name: &String, multi_path_map: &mut MultiPathMap) {
    let names: Vec<String> = multi_path_map
        .iter()
        .map(|(struct_name, _)| struct_name.clone())
        .collect();

    for struct_name in names.iter() {
        recurse_expand_multi_path(struct_name, root_struct_name, multi_path_map);
    }
}

fn recursive_rebuild_into_public(
    internal: MultiPathVector,
    all_folder_fields: &mut AllFolderFields,
) -> FolderFocusPathVector {
    internal
        .into_iter()
        .map(|multi_path| {
            let sub_paths = if multi_path.sub_paths.is_empty() {
                FolderFocusPathVector::new()
            } else {
                recursive_rebuild_into_public(multi_path.sub_paths, all_folder_fields)
            };

            let folder_info = all_folder_fields
                .entry(multi_path.struct_name.clone())
                .or_insert(FolderInfo {
                    abs_folder_key_enum: format_ident!(
                        "{}AbsPath",
                        to_camel_case(&multi_path.struct_name)
                    ),
                    folder_enum_name: format_ident!(
                        "{}Focus",
                        to_camel_case(&multi_path.struct_name)
                    ),
                    folder_focus_variable: format_ident!(
                        "focused_{}",
                        to_snake_case(&multi_path.struct_name)
                    ),
                    fields: Vec::new(),
                });

            for field_name in multi_path.parent_fields.iter() {
                let found = folder_info
                    .fields
                    .iter()
                    .find(|field_info| field_info.field_name.to_string() == *field_name);

                if found.is_none() {
                    folder_info.fields.push(FieldInfo {
                        variant_name: format_ident!("{}", to_camel_case(field_name)),
                        field_name: format_ident!("{}", field_name),
                        value_struct_name: format_ident!("{}", multi_path.struct_name),
                    });
                }
            }

            let parent_fields = multi_path
                .parent_fields
                .into_iter()
                .map(|field| format_ident!("{}", field))
                .collect();

            let multi_path = FolderFocusPath {
                struct_name: format_ident!("{}", multi_path.struct_name),
                parent_fields,
                sub_paths,
            };

            Box::new(multi_path)
        })
        .collect()
}

fn rebuild_into_public_multi_path(
    internal: MultiPathMap,
    folder_focus_path_map: &mut FolderFocusPathMap,
    all_folder_fields: &mut AllFolderFields,
) {
    for (struct_name, multi_path_vector) in internal {
        folder_focus_path_map.insert(
            struct_name,
            recursive_rebuild_into_public(multi_path_vector, all_folder_fields),
        );
    }
}

pub(super) fn build_conditional_paths(data_structure: &mut DataStructure) {
    // Build an initial map from struct names to a vector of each
    // reference to that struct
    let struct_to_reference_map = build_struct_to_reference_map(data_structure);

    // Convert the reference map into an initial struct name to multi
    // path map
    let mut multi_path_map = build_struct_to_multi_path_map(struct_to_reference_map);

    // Expand each multi path by recursively going through references to it and inserting them into
    // the sub path vector
    let root_struct = &data_structure.root_struct.as_ref().unwrap().name;
    expand_multi_path(root_struct, &mut multi_path_map);

    // Strip out fields required for building and populate the resulting folder focus path map and
    // all fields folder map in the data structure
    rebuild_into_public_multi_path(
        multi_path_map,
        &mut data_structure.folder_focus_path_map,
        &mut data_structure.all_folder_fields,
    );
}
