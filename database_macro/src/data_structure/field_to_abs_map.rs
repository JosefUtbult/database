use core::panic;
use std::collections::HashMap;

use crate::data_structure::absolute_path::compare_abs_paths;

use super::{
    absolute_path::{AbsolutePath, AbsolutePathField},
    struct_data::StructMap,
};

type FieldToAbsPathMap = HashMap<String, Vec<AbsolutePath>>;
pub(super) type FieldToAbsPathList = Vec<(String, Vec<AbsolutePath>)>;

fn recursivly_build_abs_path(
    struct_map: &StructMap,
    current_struct_name: &String,
    current_path: &AbsolutePath,
    field_to_abs_path_map: &mut FieldToAbsPathMap,
) {
    let struct_data = struct_map.get(current_struct_name).unwrap();
    for field in &struct_data.fields {
        let field_name = field.name.clone();
        let field_vector = field_to_abs_path_map
            .entry(field_name.clone())
            .or_insert(Vec::new());

        // Sanity-check all other paths
        for other_path in field_vector.iter() {
            // Verify that the type is the same as the current field
            if let Some(last) = other_path.last() {
                match last {
                    AbsolutePathField::Struct((_, other_struct_data)) => {
                        if other_struct_data.name != field.ty_string {
                            panic!(
                                "{} is mapped to multiple types: {} and {}",
                                field_name, other_struct_data.name, field.ty_string
                            )
                        }
                    }
                    AbsolutePathField::NonStruct(other_field_data) => {
                        if other_field_data.ty_string != field.ty_string {
                            panic!(
                                "{} is mapped to multiple types: {} and {}",
                                field_name, other_field_data.ty_string, field.ty_string
                            )
                        }
                    }
                }
            } else {
                panic!("Got empty field")
            }

            // Verify that if any parent are present, that parent is the same type
            // as the current struct
            if let Some(last_parent) = other_path.iter().nth_back(1) {
                match last_parent {
                    AbsolutePathField::NonStruct(field) => {
                        panic!("Got non struct parent {}", field.name)
                    }
                    AbsolutePathField::Struct((_, other_data)) => {
                        if other_data.name != struct_data.name {
                            panic!(
                                "Found duplicate field named {} in multiple structs: {} and {}",
                                field_name, other_data.name, struct_data.name
                            );
                        }
                    }
                }
            }
        }

        let mut current_path = current_path.clone();
        if let Some(child_struct) = struct_map.get(&field.ty_string) {
            current_path.push(AbsolutePathField::Struct((
                field_name,
                child_struct.clone(),
            )));
            field_vector.push(current_path.clone());

            let child_name = child_struct.name.clone();
            recursivly_build_abs_path(
                struct_map,
                &child_name,
                &current_path,
                field_to_abs_path_map,
            )
        } else {
            current_path.push(AbsolutePathField::NonStruct(field.clone()));
            field_vector.push(current_path);
        }
    }
}

pub(super) fn build_field_to_abs_path_map<'a>(struct_map: &'a mut StructMap) {
    let struct_map_clone = struct_map.clone();
    for (struct_name, struct_data) in struct_map.iter_mut() {
        let mut field_to_abs_path_map = FieldToAbsPathMap::new();

        recursivly_build_abs_path(
            &struct_map_clone,
            struct_name,
            &Vec::new(),
            &mut field_to_abs_path_map,
        );

        let mut abs_path_count = 0;
        let mut abs_folder_count = 0;

        // Then, filter out all paths that ends in structs
        for (field_name, mut abs_paths) in field_to_abs_path_map {
            if abs_paths.is_empty() {
                continue;
            }

            // Sort the path
            abs_paths.sort_by(|lhs, rhs| compare_abs_paths(&lhs, &rhs));

            // Split the paths into a vector of fields and a vector of folders
            let (field_paths, folder_paths): (Vec<AbsolutePath>, Vec<AbsolutePath>) = abs_paths
                .into_iter()
                .filter(|abs_path| !abs_path.is_empty())
                .partition(|abs_field| match abs_field.last().unwrap() {
                    AbsolutePathField::NonStruct(_) => true,
                    AbsolutePathField::Struct(_) => false,
                });

            if !field_paths.is_empty() {
                abs_path_count += field_paths.len();
                struct_data
                    .field_to_field_abs_path_map
                    .push((field_name.clone(), field_paths));
            }

            if !folder_paths.is_empty() {
                abs_folder_count += folder_paths.len();
                struct_data
                    .field_to_folder_abs_path_map
                    .push((field_name, folder_paths));
            }
        }

        // Sort both vectors on field names
        struct_data
            .field_to_field_abs_path_map
            .sort_by(|(lhs_name, _), (rhs_name, _)| lhs_name.cmp(rhs_name));

        struct_data
            .field_to_folder_abs_path_map
            .sort_by(|(lhs_name, _), (rhs_name, _)| lhs_name.cmp(rhs_name));

        struct_data.abs_field_path_count = abs_path_count;
        struct_data.abs_folder_path_count = abs_folder_count;
    }
}
