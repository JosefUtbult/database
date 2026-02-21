use std::collections::HashMap;

use crate::data_structure::struct_data::StructMap;

pub(super) type FieldToChildStructMap = HashMap<String, Vec<String>>;

pub(super) fn field_to_child_struct_map(struct_map: &mut StructMap) {
    let all_struct_names: Vec<String> = struct_map.iter().map(|(name, _)| name.clone()).collect();

    for (_, struct_data) in struct_map.iter_mut() {
        for field in struct_data.fields.iter() {
            if !all_struct_names.contains(&field.ty_string) {
                continue;
            }

            let child_struct_vector = struct_data
                .field_to_child_struct_map
                .entry(field.ty_string.clone())
                .or_insert(Vec::new());

            child_struct_vector.push(field.name.clone());
        }
    }
}
