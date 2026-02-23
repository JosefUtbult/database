use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::DataStructure;

pub(crate) fn generate_database(
    crate_path: &TokenStream2,
    data_structure: &DataStructure,
) -> TokenStream2 {
    let root_struct = data_structure.root_struct.as_ref().unwrap();
    let root_struct_name = format_ident!("{}", &root_struct.name);
    let type_names = data_structure.type_names.as_ref().unwrap();

    let database_name = type_names.database_name.clone();
    let focus_handler_name = type_names.focus_handler_name.clone();

    let abs_key_enum = root_struct.type_names.abs_key_enum.clone();
    let abs_field_enum = root_struct.type_names.abs_field_enum.clone();

    let flat_key_enum = root_struct.type_names.flat_key_enum.clone();
    let flat_field_enum = root_struct.type_names.flat_field_enum.clone();

    let abs_count_name = root_struct.type_names.abs_count_name.clone();
    let flat_count_name = root_struct.type_names.flat_count_name.clone();

    quote! {
        pub type #database_name<'a, Mutex> = #crate_path::LayerDatabase<
            'a,
            Mutex,
            #root_struct_name,
            #focus_handler_name,
            #abs_key_enum,
            #abs_field_enum,
            #flat_key_enum,
            #flat_field_enum,
            #abs_count_name,
            #flat_count_name
        >;
    }
}
