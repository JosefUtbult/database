use std::fmt::Debug;

use super::struct_data::{FieldData, StructData};

#[derive(Clone)]
pub(crate) enum AbsolutePathField {
    NonStruct(FieldData),
    Struct((String, StructData)),
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

pub(crate) type AbsolutePath = Vec<AbsolutePathField>;

