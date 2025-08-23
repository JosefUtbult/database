use core::panic;
use syn::{Data, DeriveInput, Fields, Ident, Meta, NestedMeta};

use crate::Field;

pub(crate) fn extract_attributes(input: &DeriveInput) -> (Ident, Vec<Ident>) {
    // Parse attributes
    let mut database_name: Option<Ident> = None;
    let mut subsets: Vec<Ident> = Vec::new();

    for attr in &input.attrs {
        if attr.path.is_ident("name") {
            if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                if let Some(NestedMeta::Meta(Meta::Path(path))) = meta_list.nested.first() {
                    database_name = path.get_ident().cloned();
                }
            }
        } else if attr.path.is_ident("subset") {
            if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                if let Some(NestedMeta::Meta(Meta::Path(path))) = meta_list.nested.first() {
                    subsets.push(path.get_ident().unwrap().clone());
                }
            }
        }
    }

    // Default name fallback if #[name(...)] not provided
    let database_name = match database_name {
        None => {
            panic!("name for the resulting database structure needs to be supplied");
        }
        Some(name) => name,
    };

    (database_name, subsets)
}

pub(crate) fn extract_fields(input: &DeriveInput) -> Vec<Field> {
    // Extract the fields from the input
    let fields = match input.data {
        Data::Struct(ref data_struct) => &data_struct.fields,
        _ => panic!("#[derive(Database)] can only be applied to structs"),
    };

    // Collect field names and types
    let mut field_info = Vec::new();
    if let Fields::Named(named_fields) = fields {
        for field in &named_fields.named {
            let field_name = field.ident.as_ref().unwrap();
            let field_type = &field.ty;
            field_info.push(Field {
                field_name,
                field_type,
            });
        }
    }

    field_info
}
