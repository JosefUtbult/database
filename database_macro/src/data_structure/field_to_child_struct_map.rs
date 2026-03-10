use std::collections::HashMap;

use crate::data_structure::{absolute_path::{AbsolutePath, AbsolutePathField}, struct_data::StructMap};

pub(super) type FieldToFolderMap = HashMap<String, Vec<AbsolutePath>>;

pub(super) fn field_to_folder_map(struct_map: &mut StructMap) {
    for (_, struct_data) in struct_map.iter_mut() {
        let field_to_folder_map = &mut struct_data.field_to_folder_map;

        for (_field_name, abs_paths) in struct_data.field_to_folder_abs_path_map.iter() {
            for abs_path in abs_paths.iter() {
                let abs_field = abs_path.last().unwrap();
                let struct_type = match abs_field {
                    AbsolutePathField::NonStruct(_) => todo!(),
                    AbsolutePathField::Struct((_, struct_data)) => struct_data.name.clone(),
                };

                let folder_paths = field_to_folder_map.entry(struct_type).or_insert(Vec::new());
                if !folder_paths.contains(abs_path) {
                    folder_paths.push(abs_path.clone());
                }
            }

        }
    }
}
