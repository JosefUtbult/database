use core::panic;
use std::collections::HashMap;

use crate::data_structure::{
    absolute_path::AbsolutePathField, field_to_abs_map::FieldToAbsPathList, struct_data::StructMap,
};

pub(crate) type TypeToFieldMap = HashMap<String, Vec<String>>;

fn build_all_field_types(
    _struct_name: &String,
    field_to_abs_path_list: &FieldToAbsPathList,
    result_map: &mut TypeToFieldMap,
    get_field_type: &dyn Fn(&AbsolutePathField) -> Option<(String, String)>,
) {
    for (_, abs_paths) in field_to_abs_path_list {
        if let Some(abs_path) = abs_paths.first() {
            if let Some(first_field) = abs_path.first() {
                if let Some((first_field_type, first_field_name)) = get_field_type(first_field) {
                    let field_vector = result_map.entry(first_field_type).or_insert(Vec::new());
                    if !field_vector.contains(&first_field_name) {
                        field_vector.push(first_field_name);
                    }
                }
            }
        }
    }
}

pub(crate) fn build_field_maps(struct_map: &mut StructMap) {
    for (struct_name, struct_data) in struct_map {
        build_all_field_types(
            &struct_name,
            &struct_data.field_to_abs_path_map,
            &mut struct_data.type_to_field_map,
            &|path_field| match path_field {
                AbsolutePathField::NonStruct(field_data) => Some((field_data.ty_string.clone(), field_data.name.clone())),
                AbsolutePathField::Struct(_) => None,
            },
        );

        eprintln!("{} {:?}", struct_name, struct_data.child_struct_to_field_map);
    }
}
