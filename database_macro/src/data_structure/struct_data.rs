use proc_macro2::Ident;
use quote::format_ident;
use std::vec::Vec;
use std::{collections::HashMap, fmt::Debug};
use syn::ItemStruct;

use crate::casing::{to_snake_case, to_upper_snake_case};
use crate::data_structure::all_fields::{FolderToFieldMap, TypeToFieldMap};
use crate::data_structure::field_to_child_struct_map::FieldToFolderMap;

use super::field_to_abs_map::FieldToAbsPathList;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct FieldData {
    pub(crate) name: String,
    #[allow(dead_code)]
    pub(crate) ident: Ident,
    pub(crate) ty_string: String,
}

impl Debug for FieldData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "name: {}, type: {}", self.name, self.ty_string)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct TypeNames {
    pub(crate) namespace: Ident,
    pub(crate) abs_key_enum: Ident,
    pub(crate) abs_field_enum: Ident,
    pub(crate) flat_key_enum: Ident,
    pub(crate) flat_field_enum: Ident,
    pub(crate) abs_path_count_name: Ident,
    pub(crate) flat_path_count_name: Ident,
    pub(crate) abs_folder_count_name: Ident,
    pub(crate) flat_folder_count_name: Ident,
}

#[derive(Clone)]
pub(crate) struct StructData {
    pub(crate) name: String,
    pub(crate) type_names: TypeNames,
    #[allow(dead_code)]
    pub(crate) ident: Ident,
    pub(crate) item: ItemStruct,
    pub(crate) fields: Vec<FieldData>,
    pub(crate) field_to_field_abs_path_map: FieldToAbsPathList,
    pub(crate) field_to_folder_abs_path_map: FieldToAbsPathList,
    pub(crate) type_to_field_map: TypeToFieldMap,
    pub(crate) folder_to_field_map: FolderToFieldMap,
    pub(crate) field_to_folder_map: FieldToFolderMap,
    pub(crate) abs_field_path_count: usize,
    pub(crate) abs_folder_path_count: usize,
    pub(crate) has_debug_derive: bool,
}

impl PartialEq for StructData {
    fn eq(&self, other: &Self) -> bool {
        self.name.to_string() == other.name.to_string()
    }
}

impl Eq for StructData {}

impl Debug for StructData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "name: {}, fields: {:?}", self.name.clone(), self.fields)
    }
}

pub(crate) type StructMap = HashMap<String, StructData>;

fn has_debug_derive(item_struct: &ItemStruct) -> bool {
    let attrs = item_struct.clone().attrs;
    attrs.iter().any(|attr| {
        if attr.path.is_ident("derive") {
            attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            )
            .map(|paths| paths.iter().any(|p| p.is_ident("Debug")))
            .unwrap_or(false)
        } else {
            false
        }
    })
}

pub(super) fn populate_struct_map(
    struct_names: &mut Vec<String>,
    struct_map: &mut StructMap,
    structs: Vec<ItemStruct>,
) {
    let res: Vec<StructData> = structs
        .into_iter()
        .map(|item_struct| {
            let name = item_struct.ident.to_string();
            let snake_case_name = to_snake_case(&name);
            let uppercase_name = to_upper_snake_case(&name);
            let has_debug_derive = has_debug_derive(&item_struct);
            StructData {
                type_names: TypeNames {
                    namespace: format_ident!("{}", snake_case_name.clone()),
                    abs_key_enum: format_ident!("Key"),
                    abs_field_enum: format_ident!("Field"),
                    flat_key_enum: format_ident!("FlatKey"),
                    flat_field_enum: format_ident!("FlatField"),
                    abs_path_count_name: format_ident!("{}_ABS_COUNT", uppercase_name),
                    flat_path_count_name: format_ident!("{}_FLAT_COUNT", uppercase_name),
                    abs_folder_count_name: format_ident!("{}_FOLDER_ABS_COUNT", uppercase_name),
                    flat_folder_count_name: format_ident!("{}_FOLDER_FLAT_COUNT", uppercase_name),
                },
                name,
                ident: item_struct.ident.clone(),
                item: item_struct,
                fields: Vec::new(),
                field_to_field_abs_path_map: FieldToAbsPathList::new(),
                field_to_folder_abs_path_map: FieldToAbsPathList::new(),
                type_to_field_map: TypeToFieldMap::new(),
                folder_to_field_map: FolderToFieldMap::new(),
                field_to_folder_map: FieldToFolderMap::new(),
                abs_field_path_count: 0,
                abs_folder_path_count: 0,
                has_debug_derive,
            }
        })
        .collect();

    for item in res {
        let name = item.name.clone();
        struct_names.push(name.clone());
        let _ = struct_map.insert(name, item);
    }
}
