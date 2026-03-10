use crate::{
    ParsedInput,
    casing::to_camel_case,
    data_structure::{
        all_fields::build_field_maps,
        conditional_paths::{AllFolderFields, FolderFocusPathMap, build_conditional_paths},
    },
};
use proc_macro2::Ident;
use quote::format_ident;
use std::vec::Vec;

use super::{
    build_struct_fields::build_struct_fields,
    field_to_abs_map::build_field_to_abs_path_map,
    field_to_child_struct_map::field_to_folder_map,
    find_root_struct::find_root_struct,
    struct_data::{StructData, StructMap, populate_struct_map},
};

pub(crate) struct TypeNames {
    pub(crate) database_name: Ident,
    pub(crate) focus_handler_name: Ident,
}

pub(crate) struct DataStructure {
    pub(crate) struct_map: StructMap,
    pub(crate) struct_names: Vec<String>,
    pub(crate) root_struct: Option<StructData>,
    pub(crate) type_names: Option<TypeNames>,
    pub(crate) all_structs_has_debug_derive: bool,
    pub(crate) folder_focus_path_map: FolderFocusPathMap,
    pub(crate) all_folder_fields: AllFolderFields,
}

impl DataStructure {
    pub(crate) fn new() -> Self {
        Self {
            struct_map: StructMap::new(),
            struct_names: Vec::new(),
            root_struct: None,
            type_names: None,
            all_structs_has_debug_derive: false,
            folder_focus_path_map: FolderFocusPathMap::new(),
            all_folder_fields: AllFolderFields::new(),
        }
    }
}

fn check_all_structs_has_debug(data_structure: &mut DataStructure) {
    let mut found_non_debug = false;
    for (_, struct_data) in data_structure.struct_map.iter() {
        if !struct_data.has_debug_derive {
            found_non_debug = true;
            break;
        }
    }

    data_structure.all_structs_has_debug_derive = !found_non_debug;
}

fn get_type_names(data_structure: &mut DataStructure) {
    let root_struct = data_structure.root_struct.as_ref().unwrap();

    let type_names = TypeNames {
        database_name: format_ident!("{}Database", to_camel_case(&root_struct.name)),
        focus_handler_name: format_ident!("{}Focus", to_camel_case(&root_struct.name)),
    };

    let _ = data_structure.type_names.insert(type_names);
}

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

    // Build other maps
    build_field_maps(&mut data_structure.struct_map);
    field_to_folder_map(&mut data_structure.struct_map);

    // Check if all internal structs derive debug
    check_all_structs_has_debug(&mut data_structure);

    // Locate the root struct
    find_root_struct(&mut data_structure);

    // Set the database name
    get_type_names(&mut data_structure);

    // Build the map field name -> conditional field path
    build_conditional_paths(&mut data_structure);

    data_structure
}
