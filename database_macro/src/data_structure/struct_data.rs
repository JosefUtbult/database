use proc_macro2::Ident;
use quote::{ToTokens, format_ident};
use std::vec::Vec;
use std::{collections::HashMap, fmt::Debug};
use syn::{ItemStruct, Type};

use crate::casing::to_upper_snake_case;
use crate::data_structure::field_to_child_struct_map::FieldToChildStructMap;

use super::field_to_abs_map::FieldToAbsPathMap;

#[derive(Clone)]
pub(crate) struct FieldData {
    pub(crate) name: String,
    #[allow(dead_code)]
    pub(crate) ident: Ident,
    pub(crate) ty_string: String,
    pub(crate) ty: Type,
}

impl Debug for FieldData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "name: {}, type: {}",
            self.name,
            self.ty.clone().into_token_stream().to_string()
        )
    }
}

#[derive(Clone)]
pub(crate) struct TypeNames {
    pub(crate) abs_key_enum: Ident,
    pub(crate) abs_field_enum: Ident,
    pub(crate) flat_key_enum: Ident,
    pub(crate) flat_field_enum: Ident,
    #[allow(dead_code)]
    pub(crate) abs_count_name: Ident,
    #[allow(dead_code)]
    pub(crate) flat_count_name: Ident,
    #[allow(dead_code)]
    pub(crate) folder_key_name: Ident,
    #[allow(dead_code)]
    pub(crate) folder_field_name: Ident,
    #[allow(dead_code)]
    pub(crate) folder_mut_field_name: Ident,
}

#[derive(Clone)]
pub(crate) struct StructData {
    pub(crate) name: String,
    pub(crate) type_names: TypeNames,
    #[allow(dead_code)]
    pub(crate) ident: Ident,
    pub(crate) item: ItemStruct,
    pub(crate) fields: Vec<FieldData>,
    pub(crate) field_to_abs_path_map: FieldToAbsPathMap,
    pub(crate) child_struct_to_abs_path_map: FieldToAbsPathMap,
    pub(crate) field_to_child_struct_map: FieldToChildStructMap,
    pub(crate) abs_path_count: usize,
    pub(crate) has_debug_derive: bool,
}

impl Debug for StructData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "name: {}, fields: {:?}", self.name.clone(), self.fields)
    }
}

impl PartialEq for StructData {
    fn eq(&self, other: &Self) -> bool {
        self.name.to_string() == other.name.to_string()
    }
}

pub(super) type StructMap = HashMap<String, StructData>;

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
            let uppercase_name = to_upper_snake_case(&name);
            let has_debug_derive = has_debug_derive(&item_struct);
            StructData {
                type_names: TypeNames {
                    abs_key_enum: format_ident!("{}AbsKey", name.clone()),
                    abs_field_enum: format_ident!("{}AbsField", name.clone()),
                    flat_key_enum: format_ident!("{}Key", name.clone()),
                    flat_field_enum: format_ident!("{}Field", name.clone()),
                    folder_key_name: format_ident!("{}FolderKey", name.clone()),
                    folder_field_name: format_ident!("{}FolderKey", name.clone()),
                    folder_mut_field_name: format_ident!("{}FolderFieldMut", name.clone()),
                    abs_count_name: format_ident!("{}_ABS_COUNT", uppercase_name),
                    flat_count_name: format_ident!("{}_FLAT_COUNT", uppercase_name),
                },
                name,
                ident: item_struct.ident.clone(),
                item: item_struct,
                fields: Vec::new(),
                field_to_abs_path_map: FieldToAbsPathMap::new(),
                child_struct_to_abs_path_map: FieldToAbsPathMap::new(),
                field_to_child_struct_map: FieldToChildStructMap::new(),
                abs_path_count: 0,
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
