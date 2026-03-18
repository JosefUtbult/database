use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::DataStructure;

pub(crate) fn generate_database(
    crate_path: &TokenStream2,
    data_structure: &DataStructure,
) -> TokenStream2 {
    let root_struct = data_structure.root_struct.as_ref().unwrap();
    let root_struct_name = format_ident!("{}", &root_struct.name);
    let root_struct_description = format_ident!("{}Description", &root_struct.name);
    let type_names = data_structure.type_names.as_ref().unwrap();

    let database_name = type_names.database_name.clone();
    let focus_handler_name = type_names.focus_handler_name.clone();

    let abs_key_enum = root_struct.type_names.abs_key_enum.clone();
    let abs_field_enum = root_struct.type_names.abs_field_enum.clone();

    let flat_key_enum = root_struct.type_names.flat_key_enum.clone();
    let flat_field_enum = root_struct.type_names.flat_field_enum.clone();

    let flat_count_name = root_struct.type_names.flat_path_count_name.clone();

    quote! {
        pub struct #root_struct_description {}
        impl #crate_path::DatabaseDescription for #root_struct_description {
            type AbsKey = #abs_key_enum;
            type AbsField = #abs_field_enum;
            type FlatKey = #flat_key_enum;
            type FlatField = #flat_field_enum;
            type Data = #root_struct_name;
        }

        pub type #database_name<'a, Mutex> = #crate_path::LayerDatabase<
            'a,
            Mutex,
            #focus_handler_name,
            #root_struct_description,
            #flat_count_name
        >;
    }
}
