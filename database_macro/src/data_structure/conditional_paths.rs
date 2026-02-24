use core::{fmt, panic};
use std::fmt::Write;
use std::hint::unreachable_unchecked;
use std::{cmp::min, collections::HashMap};

use crate::{
    DataStructure,
    data_structure::absolute_path::{AbsolutePath, AbsolutePathField},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Conditional {
    name: String,
    conditions: Vec<(String, Box<ConditionalPath>)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ConditionalPathField {
    NonConditional(AbsolutePathField),
    Conditional(Box<Conditional>),
}

pub(crate) struct ConditionalFieldInfo {
    conditional_map: ConditionalFieldToAbsPathMap,
    #[allow(dead_code)]
    all_conditionals: AllConditionals,
}

pub(crate) type ConditionalPath = Vec<ConditionalPathField>;

fn replace_space_at(line: &mut String, index: usize, new: char) {
    let mut iter = line.char_indices();
    if let Some((start, character)) = iter.nth(index) {
        if character == ' ' {
            let end = iter.next().map(|(i, _)| i).unwrap_or_else(|| line.len());
            line.replace_range(start..end, &new.to_string());
        }
    }
}

fn format_path_recursive(path: &ConditionalPath, lines: &mut Vec<String>) -> std::fmt::Result {
    let buffer = lines.last_mut().unwrap();
    for field in path.iter() {
        match field {
            ConditionalPathField::NonConditional(absolute_path_field) => {
                let (field_name, field_type) = absolute_path_field.name_type_pair();
                write!(buffer, "--{}::{}", field_name, field_type)?;
            }
            ConditionalPathField::Conditional(conditional) => {
                write!(buffer, "--({})", conditional.name)?;
                let current_width = buffer.len();
                if let Some((first_field, first_path)) = conditional.conditions.first() {
                    write!(buffer, "--[{}]", first_field)?;
                    format_path_recursive(&first_path, lines)?;

                    for (next_field, next_path) in &conditional.conditions[1..] {
                        replace_space_at(lines.last_mut().unwrap(), current_width - 2, '|');

                        lines.push(String::new());
                        let buffer = lines.last_mut().unwrap();
                        buffer.push_str(&" ".repeat(current_width - 1));
                        buffer.push_str("\\");

                        write!(buffer, "--[{}]", next_field)?;
                        format_path_recursive(&next_path, lines)?;
                    }
                }

                break;
            }
        }
    }
    Ok(())
}

fn print_path(field: &String, path: &ConditionalPath) {
    let mut lines: Vec<String> = Vec::new();
    lines.push(field.clone());
    match format_path_recursive(path, &mut lines) {
        Err(_) => eprintln!("Unable to format path {:?}", path),
        Ok(_) => {
            for line in lines.iter() {
                eprintln!("{}", line);
            }
        }
    }
}

// impl fmt::Debug for ConditionalPath {
// }

impl fmt::Debug for ConditionalFieldInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.conditional_map.fmt(f)
    }
}

impl ConditionalFieldInfo {
    pub(super) fn new() -> Self {
        Self {
            conditional_map: ConditionalFieldToAbsPathMap::new(),
            all_conditionals: AllConditionals::new(),
        }
    }
}

pub(crate) type ConditionalFieldToAbsPathMap = HashMap<String, ConditionalPath>;
pub(crate) type AllConditionals = HashMap<String, Vec<String>>;

fn abs_path_to_non_conditional_path(abs_path: &[AbsolutePathField]) -> ConditionalPath {
    abs_path
        .iter()
        .map(|abs_field| ConditionalPathField::NonConditional(abs_field.clone()))
        .collect()
}

fn get_struct_name_from_abs_path(path: &[AbsolutePathField]) -> Option<String> {
    for field in path.iter().rev() {
        match field {
            AbsolutePathField::NonStruct(_) => continue,
            AbsolutePathField::Struct((_, struct_data)) => return Some(struct_data.name.clone()),
        }
    }
    None
}

fn push_conditional(
    condition_name: String,
    condition_value: String,
    all_conditionals: &mut AllConditionals,
) {
    let condition_vector = all_conditionals
        .entry(condition_name.clone())
        .or_insert(Vec::new());
    if !condition_vector.contains(&condition_value) {
        condition_vector.push(condition_value);
    }
}

// Note: This assumes that the conditional path length is >= then the absolute path
fn insert_abs_path_into_conditional(
    conditional_path: &mut ConditionalPath,
    all_conditionals: &mut AllConditionals,
    abs_path: &[AbsolutePathField],
) {
    for index in 0..abs_path.len() {
        let abs_field = &abs_path[index];
        let (abs_field_name, _abs_field_type) = abs_field.name_type_pair();

        match conditional_path[index].clone() {
            ConditionalPathField::NonConditional(conditional_abs_path_field) => {
                if conditional_abs_path_field == *abs_field {
                    continue;
                }

                // Rebuild every element in the tail up to this index into a new path
                let mut tail = ConditionalPath::new();
                while conditional_path.len() > index {
                    tail.push(conditional_path.pop().unwrap());
                }
                tail.reverse();

                let (tail_field, _) = conditional_abs_path_field.name_type_pair();
                let tail_condition = (tail_field.clone(), Box::new(tail));

                let new_non_conditional_path = abs_path_to_non_conditional_path(&abs_path[index..]);
                let new_condition = (abs_field_name.clone(), Box::new(new_non_conditional_path));

                let conditions = vec![tail_condition, new_condition];
                let abs_field_struct_name = get_struct_name_from_abs_path(&abs_path).unwrap();

                // Now we start creating conditionals
                let conditional = Conditional {
                    name: abs_field_struct_name.clone(),
                    conditions,
                };

                push_conditional(abs_field_struct_name.clone(), tail_field, all_conditionals);
                push_conditional(
                    abs_field_struct_name.clone(),
                    abs_field_name,
                    all_conditionals,
                );

                conditional_path.push(ConditionalPathField::Conditional(Box::new(conditional)));
                return;
            }
            ConditionalPathField::Conditional(_) => {
                let conditions = match &mut conditional_path[index] {
                    ConditionalPathField::NonConditional(_) => unreachable!(),
                    ConditionalPathField::Conditional(conditional) => &mut conditional.conditions,
                };

                // Check if this is an existing condition based on the unique name
                if let Some((_, sub_conditional)) = conditions
                    .iter_mut()
                    .find(|(name, _)| *name == abs_field_name)
                {
                    // Recurse into this conditions
                    insert_abs_path_into_conditional(sub_conditional, all_conditionals, &abs_path);
                }
                // Otherwise, insert a totally new condition
                else {
                    let new_non_conditional_path =
                        abs_path_to_non_conditional_path(&abs_path[index..]);
                    let new_condition =
                        (abs_field_name.clone(), Box::new(new_non_conditional_path));

                    conditions.push(new_condition);

                    let abs_field_struct_name = get_struct_name_from_abs_path(&abs_path).unwrap();
                    push_conditional(
                        abs_field_struct_name,
                        abs_field_name.clone(),
                        all_conditionals,
                    );
                }

                return;
            }
        }
    }
}

#[allow(dead_code)]
fn build_conditional(
    abs_paths: &Vec<AbsolutePath>,
    all_conditionals: &mut AllConditionals,
) -> Option<ConditionalPath> {
    if abs_paths.len() > 1 {
        // Find the index of the longest path in the list of abs paths
        let longest_abs_path_index = abs_paths
            .iter()
            .enumerate()
            .max_by_key(|(_, abs_path)| abs_path.len())
            .map(|(i, _)| i)
            .unwrap();

        let longest_abs_path = &abs_paths[longest_abs_path_index];

        // Create an initial conditional path
        let mut conditional_path = abs_path_to_non_conditional_path(longest_abs_path);

        // Iterate through all other abs paths
        for index in 0..abs_paths.len() {
            if index == longest_abs_path_index {
                continue;
            }

            let next_abs_path = &abs_paths[index];
            insert_abs_path_into_conditional(
                &mut conditional_path,
                all_conditionals,
                next_abs_path,
            );
        }

        Some(conditional_path)
    } else {
        None
    }
}

pub(super) fn build_conditional_paths(data_structure: &mut DataStructure) {
    let root_struct = data_structure.root_struct.as_ref().unwrap();
    let field_to_abs_map = &root_struct.field_to_abs_path_map;
    let conditional_field_info = &mut data_structure.conditional_field_info;

    let mut all_conditionals = AllConditionals::new();

    for (field_name, abs_paths) in field_to_abs_map.iter() {
        if let Some(conditional_path) = build_conditional(abs_paths, &mut all_conditionals) {
            print_path(&field_name, &conditional_path);
            conditional_field_info
                .conditional_map
                .insert(field_name.clone(), conditional_path);
        }
    }
}
