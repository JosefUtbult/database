use core::panic;
use proc_macro2::Ident;
use quote::ToTokens;
use std::fmt::Debug;
use std::{collections::HashMap, vec::Vec};
use syn::{ItemStruct, Type};

use crate::ParsedInput;

#[derive(Clone)]
pub(crate) struct FieldData {
    name: String,
    #[allow(dead_code)]
    ident: Ident,
    ty_string: String,
    ty: Type,
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
pub(crate) struct StructData {
    name: String,
    #[allow(dead_code)]
    ident: Ident,
    item: ItemStruct,
    fields: Vec<FieldData>,
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

#[derive(Clone)]
pub(crate) enum AbsolutePathField<'a> {
    NonStruct(&'a FieldData),
    Struct((String, &'a StructData)),
}

impl<'a> Debug for AbsolutePathField<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AbsolutePathField::NonStruct(field) => {
                write!(
                    f,
                    "AbsolutePathField::Field({}({}))",
                    field.name, field.ty_string
                )
            }
            AbsolutePathField::Struct(struct_data) => {
                write!(
                    f,
                    "AbsolutePathField::Struct({}({}))",
                    struct_data.0, struct_data.1.name
                )
            }
        }
    }
}

type AbsolutePath<'a> = Vec<AbsolutePathField<'a>>;
type FieldToAbsPathMap<'a> = HashMap<String, Vec<AbsolutePath<'a>>>;

#[derive(Debug, Clone)]
struct ConditionalPath<'a> {
    name: String,
    path: &'a AbsolutePath<'a>,
}

type ConditionalFieldToAbsPathMap<'a> = HashMap<String, Vec<ConditionalPath<'a>>>;

type StructMap<'a> = HashMap<String, &'a StructData>;

pub(crate) struct DataStructure<'a> {
    root_name: Option<Ident>,
    struct_data: Vec<StructData>,
    struct_map: StructMap<'a>,
    struct_names: Vec<String>,
    root_struct: Option<&'a StructData>,
    field_to_abs_path_map: FieldToAbsPathMap<'a>,
    conditional_field_to_abs_path_map: ConditionalFieldToAbsPathMap<'a>,
}

impl<'a> DataStructure<'a> {
    pub(crate) fn new() -> Self {
        Self {
            root_name: None,
            struct_data: Vec::new(),
            struct_map: StructMap::new(),
            struct_names: Vec::new(),
            root_struct: None,
            field_to_abs_path_map: FieldToAbsPathMap::new(),
            conditional_field_to_abs_path_map: ConditionalFieldToAbsPathMap::new(),
        }
    }
}

fn populate_struct_data(struct_data: &mut Vec<StructData>, structs: Vec<ItemStruct>) {
    let res: Vec<StructData> = structs
        .into_iter()
        .map(|item_struct| StructData {
            name: item_struct.ident.to_string(),
            ident: item_struct.ident.clone(),
            item: item_struct,
            fields: Vec::new(),
        })
        .collect();

    for item in res {
        struct_data.push(item);
    }
}

fn build_struct_fields<'a>(struct_data: &'a mut Vec<StructData>) {
    for struct_info in struct_data.iter_mut() {
        match &struct_info.item.fields {
            syn::Fields::Named(fields) => {
                for field in fields.named.iter() {
                    let field_ident = field.ident.as_ref().unwrap();
                    let field = FieldData {
                        name: field_ident.to_string(),
                        ident: field_ident.clone(),
                        ty_string: field.ty.clone().to_token_stream().to_string(),
                        ty: field.ty.clone(),
                    };

                    struct_info.fields.push(field);
                }
            }
            _ => panic!("Only named fields are supported"),
        }
    }
}

fn populate_struct_map<'a>(
    struct_map: &mut StructMap<'a>,
    struct_names: &mut Vec<String>,
    struct_data: &'a Vec<StructData>,
) {
    for struct_instance in struct_data.iter() {
        struct_map.insert(struct_instance.name.clone(), struct_instance);
        struct_names.push(struct_instance.name.clone());
    }
}

fn find_root_struct<'a>(struct_map: &StructMap<'a>) -> &'a StructData {
    let mut root_elements: Vec<&'a StructData> = Vec::new();
    for (potential_root_struct_name, potential_root_struct_data) in struct_map {
        let mut found_parent = false;
        for (compared_struct_name, compared_struct_data) in struct_map {
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
            root_elements.push(potential_root_struct_data);
        }
    }

    if root_elements.len() == 0 {
        panic!("No root struct found")
    } else if root_elements.len() != 1 {
        panic!("Found multiple root structs")
    }

    root_elements.first().unwrap()
}

fn recursivly_build_abs_path<'a>(
    current_path: &AbsolutePath<'a>,
    struct_data: &'a StructData,
    field_to_abs_path_map: &mut FieldToAbsPathMap<'a>,
    struct_map: &'a StructMap<'a>,
) {
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
            current_path.push(AbsolutePathField::Struct((field_name, child_struct)));
            field_vector.push(current_path.clone());

            recursivly_build_abs_path(
                &current_path,
                child_struct,
                field_to_abs_path_map,
                struct_map,
            )
        } else {
            current_path.push(AbsolutePathField::NonStruct(field));
            field_vector.push(current_path);
        }
    }
}

fn build_field_to_abs_path_map<'a>(
    root_struct: &'a StructData,
    field_to_abs_path_map: &mut FieldToAbsPathMap<'a>,
    struct_map: &'a StructMap<'a>,
) {
    recursivly_build_abs_path(&Vec::new(), root_struct, field_to_abs_path_map, struct_map);

    // Now filter out all fields ending with a struct. These are added from the start in order to
    // verify all field types
    field_to_abs_path_map.retain(|_, abs_path| {
        if let Some(last_path) = abs_path.last() {
            if let Some(last_field) = last_path.last() {
                match last_field {
                    AbsolutePathField::NonStruct(_) => true,
                    AbsolutePathField::Struct(_) => false,
                }
            }
            // Ignore empty paths
            else {
                false
            }
        }
        // Ignore empty lists of paths
        else {
            false
        }
    });

    eprintln!("{:?}", field_to_abs_path_map);
}

fn find_differing_field<'a>(lhs: &AbsolutePath<'a>, rhs: &AbsolutePath<'a>) -> (String, String) {
    for lhs_field in lhs.iter().rev() {
        match lhs_field {
            AbsolutePathField::NonStruct(_) => {}
            AbsolutePathField::Struct((lhs_field_name, lhs_struct_data)) => {
                for rhs_field in rhs.iter().rev() {
                    match rhs_field {
                        AbsolutePathField::NonStruct(_) => {}
                        AbsolutePathField::Struct((rhs_field_name, rhs_struct_data)) => {
                            if lhs_struct_data.name == rhs_struct_data.name {
                                return (lhs_field_name.clone(), rhs_field_name.clone())
                            }
                        }
                    }
                }
            }
        }
    }

    panic!("Unable to find differing parameter name");
}

fn build_conditional_field_to_abs_path_map<'a>(
    conditional_field_to_abs_path_map: &mut ConditionalFieldToAbsPathMap<'a>,
    field_to_abs_path_map: &'a FieldToAbsPathMap<'a>,
) {
    for (field_name, field_paths) in field_to_abs_path_map {
        if field_paths.len() <= 1 {
            continue;
        }

        let mut conditional_paths: Vec<ConditionalPath<'a>> = Vec::new();
        let first_path = field_paths.first().unwrap();
        for path in &field_paths[1..] {
            let (first_differing_field, second_differing_field) = find_differing_field(first_path, path);

            if conditional_paths.is_empty() {
                conditional_paths.push(ConditionalPath {
                    name: first_differing_field,
                    path: first_path
                });
            }

            conditional_paths.push(ConditionalPath {
                name: second_differing_field,
                path
            });
        }

        conditional_field_to_abs_path_map.insert(field_name.clone(), conditional_paths);
    }

    eprintln!("Conditional paths: {:?}", conditional_field_to_abs_path_map);
}

#[allow(dead_code)]
pub(crate) fn populate_data_structure<'a>(
    data_structure: &'a mut DataStructure<'a>,
    parsed_input: ParsedInput,
) {
    // Set the root name
    let _ = data_structure.root_name.insert(parsed_input.name);

    // Populate the vector of structs
    populate_struct_data(&mut data_structure.struct_data, parsed_input.structs);

    // Build the structs field list
    build_struct_fields(&mut data_structure.struct_data);

    // Build a map of all structs, and a list off all struct names
    populate_struct_map(
        &mut data_structure.struct_map,
        &mut data_structure.struct_names,
        &data_structure.struct_data,
    );

    // Locate the root struct
    let _ = data_structure
        .root_struct
        .insert(find_root_struct(&data_structure.struct_map));

    // Build the map field name -> absolute field path
    build_field_to_abs_path_map(
        &data_structure.root_struct.unwrap(),
        &mut data_structure.field_to_abs_path_map,
        &data_structure.struct_map,
    );

    // Build the map field name -> conditional field path
    build_conditional_field_to_abs_path_map(
        &mut data_structure.conditional_field_to_abs_path_map,
        &data_structure.field_to_abs_path_map,
    );
}
