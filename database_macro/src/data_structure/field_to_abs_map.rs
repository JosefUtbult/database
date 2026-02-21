use core::panic;
use std::collections::HashMap;

use super::{
    absolute_path::{AbsolutePath, AbsolutePathField},
    struct_data::StructMap,
};

pub(super) type FieldToAbsPathMap = HashMap<String, Vec<AbsolutePath>>;

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

        // Then, filter out all paths that ends in structs
        for (field_name, abs_paths) in field_to_abs_path_map {
            if abs_paths.is_empty() {
                continue;
            }

            // Technically, all paths should be the same type. But just to make sure, explicitly
            // check every instance
            let mut struct_paths = Vec::new();
            let mut non_struct_paths = Vec::new();
            for abs_path in abs_paths {
                if let Some(last_path) = abs_path.last() {
                    match last_path {
                        AbsolutePathField::Struct(_) => {
                            struct_paths.push(abs_path);
                        }
                        AbsolutePathField::NonStruct(_) => {
                            non_struct_paths.push(abs_path);
                        }
                    }
                }
            }

            if !(struct_paths.is_empty() || non_struct_paths.is_empty()) {
                eprintln!("Got paths for the same variable both as structs and non structs");
                eprintln!("Structs: {:?}", struct_paths);
                eprintln!("Non Structs: {:?}", non_struct_paths);
                panic!()
            }

            if !struct_paths.is_empty() {
                struct_data
                    .child_struct_to_abs_path_map
                    .insert(field_name, non_struct_paths);
            } else if !non_struct_paths.is_empty() {
                struct_data
                    .field_to_abs_path_map
                    .insert(field_name, non_struct_paths);
            }
        }
    }
}
