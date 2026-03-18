use quote::format_ident;

use crate::DataStructure;

use super::struct_data::StructData;

pub(super) fn find_root_struct(data_structure: &mut DataStructure) {
    let mut root_elements: Vec<String> = Vec::new();
    for (potential_root_struct_name, potential_root_struct_data) in data_structure.struct_map.iter()
    {
        let mut found_parent = false;
        for (compared_struct_name, compared_struct_data) in data_structure.struct_map.iter() {
            if compared_struct_name == potential_root_struct_name {
                continue;
            }

            for field in compared_struct_data.fields.iter() {
                if field.ty_string == *potential_root_struct_name {
                    found_parent = true;
                    break;
                }
            }

            if found_parent {
                break;
            }
        }

        if !found_parent {
            root_elements.push(potential_root_struct_name.clone());
        }
    }

    if root_elements.len() == 0 {
        panic!("No root struct found")
    } else if root_elements.len() != 1 {
        panic!("Found multiple root structs")
    }

    let root_struct = data_structure.struct_map.get_mut(root_elements.first().unwrap()).unwrap();

    // Rename the enum types
    root_struct.type_names.abs_key_enum = format_ident!("AbsKey");
    root_struct.type_names.abs_field_enum = format_ident!("AbsField");
    root_struct.type_names.flat_key_enum = format_ident!("Key");
    root_struct.type_names.flat_field_enum = format_ident!("Field");

    let _ = data_structure.root_struct.insert(root_struct.clone());
}
