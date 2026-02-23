use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::{DataStructure, casing::to_camel_case};

pub(crate) fn generate_focus_handler(
    crate_path: &TokenStream2,
    data_structure: &DataStructure,
) -> TokenStream2 {
    let root_struct = data_structure.root_struct.as_ref().unwrap();
    let focus_handler_name = data_structure
        .type_names
        .as_ref()
        .unwrap()
        .focus_handler_name
        .clone();

    let abs_key_enum = root_struct.type_names.abs_key_enum.clone();
    let abs_field_enum = root_struct.type_names.abs_field_enum.clone();

    let flat_key_enum = root_struct.type_names.flat_key_enum.clone();
    let flat_field_enum = root_struct.type_names.flat_field_enum.clone();

    quote! {
        pub struct #focus_handler_name {}

        impl #crate_path::FocusHandler<#abs_key_enum, #abs_field_enum, #flat_key_enum, #flat_field_enum> for #focus_handler_name {
            fn get_focus_key(&self, key: #flat_key_enum) -> #abs_key_enum {
                todo!();
            }

            fn get_focus_field(&self, field: #flat_field_enum) -> #abs_field_enum {
                todo!();
            }
        }
    }
}
