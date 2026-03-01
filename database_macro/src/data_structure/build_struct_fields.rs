use super::struct_data::{FieldData, StructMap};
use core::panic;
use quote::ToTokens;

pub(super) fn build_struct_fields<'a>(struct_data: &mut StructMap) {
    for (_, struct_data) in struct_data.iter_mut() {
        match &struct_data.item.fields {
            syn::Fields::Named(fields) => {
                for field in fields.named.iter() {
                    let field_ident = field.ident.as_ref().unwrap();
                    let field = FieldData {
                        name: field_ident.to_string(),
                        ident: field_ident.clone(),
                        ty_string: field.ty.to_token_stream().to_string(),
                    };

                    struct_data.fields.push(field);
                }
            }
            _ => panic!("Only named fields are supported"),
        }
    }
}
