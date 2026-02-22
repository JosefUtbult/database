use std::{collections::HashMap, fmt::Debug, panic, vec::Vec};

use crate::{ParsedInput, data_structure};

use super::{
    absolute_path::{AbsolutePath, AbsolutePathField},
    build_struct_fields::build_struct_fields,
    field_to_abs_map::FieldToAbsPathMap,
    field_to_abs_map::build_field_to_abs_path_map,
    field_to_child_struct_map::field_to_child_struct_map,
    find_root_struct::find_root_struct,
    struct_data::{StructData, StructMap, populate_struct_map},
};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ConditionalPath {
    name: String,
    path: AbsolutePath,
}

pub(crate) type ConditionalFieldToAbsPathMap = HashMap<String, Vec<ConditionalPath>>;

pub(crate) struct DataStructure {
    pub(crate) struct_map: StructMap,
    pub(crate) struct_names: Vec<String>,
    pub(crate) root_struct: Option<StructData>,
    #[allow(dead_code)]
    pub(crate) conditional_field_to_abs_path_map: ConditionalFieldToAbsPathMap,
}

impl DataStructure {
    pub(crate) fn new() -> Self {
        Self {
            struct_map: StructMap::new(),
            struct_names: Vec::new(),
            root_struct: None,
            conditional_field_to_abs_path_map: ConditionalFieldToAbsPathMap::new(),
        }
    }
}

#[allow(dead_code)]
fn find_differing_field(lhs: &AbsolutePath, rhs: &AbsolutePath) -> (String, String) {
    for lhs_field in lhs.iter().rev() {
        match lhs_field {
            AbsolutePathField::NonStruct(_) => {}
            AbsolutePathField::Struct((lhs_field_name, lhs_struct_data)) => {
                for rhs_field in rhs.iter().rev() {
                    match rhs_field {
                        AbsolutePathField::NonStruct(_) => {}
                        AbsolutePathField::Struct((rhs_field_name, rhs_struct_data)) => {
                            if lhs_struct_data.name == rhs_struct_data.name {
                                return (lhs_field_name.clone(), rhs_field_name.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    panic!("Unable to find differing parameter name");
}

#[allow(dead_code)]
fn build_conditional_field_to_abs_path_map(
    conditional_field_to_abs_path_map: &mut ConditionalFieldToAbsPathMap,
    field_to_abs_path_map: &FieldToAbsPathMap,
) {
    for (field_name, field_paths) in field_to_abs_path_map {
        if field_paths.len() <= 1 {
            continue;
        }

        let mut conditional_paths: Vec<ConditionalPath> = Vec::new();
        let first_path = field_paths.first().unwrap().clone();
        for path in &field_paths[1..] {
            let (first_differing_field, second_differing_field) =
                find_differing_field(&first_path, path);

            if conditional_paths.is_empty() {
                conditional_paths.push(ConditionalPath {
                    name: first_differing_field,
                    path: first_path.clone(),
                });
            }

            conditional_paths.push(ConditionalPath {
                name: second_differing_field,
                path: path.clone(),
            });
        }

        conditional_field_to_abs_path_map.insert(field_name.clone(), conditional_paths);
    }

    eprintln!("Conditional paths: {:?}", conditional_field_to_abs_path_map);
}

#[allow(dead_code)]
pub(crate) fn build_data_structure(parsed_input: ParsedInput) -> DataStructure {
    let mut data_structure: DataStructure = DataStructure::new();

    // Populate the instal values for the vector of structs
    populate_struct_map(
        &mut data_structure.struct_names,
        &mut data_structure.struct_map,
        parsed_input.structs,
    );

    // Build the structs field list
    build_struct_fields(&mut data_structure.struct_map);

    // Build the map field name -> absolute field path
    build_field_to_abs_path_map(&mut data_structure.struct_map);

    field_to_child_struct_map(&mut data_structure.struct_map);

    // for (name, data) in data_structure.struct_map.iter() {
    //     eprintln!("{}: {:?}\n", name, data.field_to_abs_path_map)
    // }

    // Locate the root struct
    let _ = data_structure
        .root_struct
        .insert(find_root_struct(&data_structure.struct_map));

    // Build the map field name -> conditional field path
    // build_conditional_field_to_abs_path_map(
    //     &mut data_structure.conditional_field_to_abs_path_map,
    //     &data_structure.field_to_abs_path_map_vector,
    // );

    data_structure
}
