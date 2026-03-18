use std::collections::HashMap;

use crate::data_structure::{
    absolute_path::{AbsolutePathField, get_parent_struct_type},
    struct_data::StructMap,
};

pub(crate) type TypeToFieldMap = HashMap<String, Vec<String>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FieldPair {
    pub(crate) top_field: String,
    pub(crate) bottom_field: String,
}

pub(crate) type FolderToFieldMap = HashMap<String, Vec<FieldPair>>;

#[allow(dead_code)]
pub(crate) fn find_folder_type(
    field_name: &String,
    folder_to_field_map: &FolderToFieldMap,
) -> Option<String> {
    let result = folder_to_field_map.iter().find(|(_, field_vector)| {
        field_vector
            .iter()
            .find(|field_pair| {
                field_pair.top_field == *field_name || field_pair.bottom_field == *field_name
            })
            .is_some()
    });

    match result {
        Some((folder_type, _)) => Some(folder_type.clone()),
        None => None,
    }
}

pub(crate) fn build_field_maps(struct_map: &mut StructMap) {
    for (_, struct_data) in struct_map {
        // Start by finding all fields in the field to abs path map
        for (field_name, abs_paths) in struct_data.field_to_field_abs_path_map.iter() {
            if let Some(abs_path) = abs_paths.first() {
                // Begin with fields
                if let Some(first_field) = abs_path.first() {
                    // Filter out folders
                    let field_type = match first_field {
                        AbsolutePathField::NonStruct(field_data) => {
                            Some(field_data.ty_string.clone())
                        }
                        AbsolutePathField::Struct(_) => None,
                    };

                    if let Some(field_type) = field_type {
                        // find the field name in the type map and insert the field name if it
                        // doesn't already exist
                        let field_vector = struct_data
                            .type_to_field_map
                            .entry(field_type)
                            .or_insert(Vec::new());

                        if !field_vector.contains(&field_name) {
                            field_vector.push(field_name.clone());
                        }
                    }
                }

                if let Some(last_field) = abs_path.last() {
                    // Then all fields parent struct types
                    let parent_struct_type = get_parent_struct_type(abs_path);
                    let (bottom_field_name, _) = last_field.name_type_pair();
                    if let Some(parent_struct_type) = parent_struct_type {
                        // find the field name in the type map and insert the field name if it
                        // doesn't already exist
                        let field_vector = struct_data
                            .folder_to_field_map
                            .entry(parent_struct_type)
                            .or_insert(Vec::new());

                        let field_pair = FieldPair {
                            top_field: field_name.clone(),
                            bottom_field: bottom_field_name,
                        };

                        if !field_vector.contains(&field_pair) {
                            field_vector.push(field_pair);
                        }
                    }
                }
            }
        }
    }
}
