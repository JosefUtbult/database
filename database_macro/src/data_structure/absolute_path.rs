use std::{cmp::Ordering, fmt::Debug};

use super::struct_data::{FieldData, StructData};

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum AbsolutePathField {
    NonStruct(FieldData),
    Struct((String, StructData)),
}

impl AbsolutePathField {
    #[allow(dead_code)]
    pub(crate) fn name_type_pair(&self) -> (String, String) {
        match self {
            AbsolutePathField::NonStruct(field_data) => {
                (field_data.name.clone(), field_data.ty_string.clone())
            }
            AbsolutePathField::Struct((field_name, struct_data)) => {
                (field_name.clone(), struct_data.name.clone())
            }
        }
    }
}

impl Debug for AbsolutePathField {
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

pub(crate) fn compare_abs_paths(
    lhs_abs_path: &[AbsolutePathField],
    rhs_abs_path: &[AbsolutePathField],
) -> Ordering {
    let lhs_first = lhs_abs_path.first();
    let rhs_first = rhs_abs_path.first();
    if let Some(lhs_first) = lhs_first
        && let Some(rhs_first) = rhs_first
    {
        if lhs_first == rhs_first {
            compare_abs_paths(&lhs_abs_path[1..], &rhs_abs_path[1..])
        } else {
            let (lhs_field_name, _) = lhs_first.name_type_pair();
            let (rhs_field_name, _) = rhs_first.name_type_pair();

            if lhs_field_name < rhs_field_name {
                Ordering::Less
            } else if lhs_field_name == rhs_field_name {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        }
    } else if let Some(_) = lhs_first {
        Ordering::Less
    } else if let Some(_) = rhs_first {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

pub(crate) type AbsolutePath = Vec<AbsolutePathField>;
