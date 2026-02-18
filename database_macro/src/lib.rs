mod parse_input;
mod data_structure;

// mod build_absolut_enums;
// mod build_parameter_type_lists;

// mod build_flat_enums;

// mod casing;
// use casing::*;
use syn::parse_macro_input;

mod base_structs;

use core::panic;
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::{
    data_structure::{populate_data_structure, DataStructure}, parse_input::ParsedInput
};

const CRATE_NAME: &str = "database";

#[proc_macro]
pub fn build_database(input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as ParsedInput);
    let res = TokenStream2::new();

    let mut data_structure = DataStructure::new();
    populate_data_structure(&mut data_structure, parsed_input);

//     let (inheritence_map, root_struct_name) = map_struct_inheritence(&parsed_input.structs);
//     // eprintln!("Inheritence: {:?}", inheritence_map);

//     let flattened_fields = build_flattened_field_map(&parsed_input.structs);

//     // Add back the input structs to the output stream
//     {
//         let base_structs = rebuild_structs(&parsed_input);
//         res.extend(base_structs);
//     }

//     // Build flat enums for all database fields
//     {
//         let flat_enums = build_flat_enums(&parsed_input, &flattened_fields);
//         res.extend(flat_enums);
//     }

//     // Build abs enums for all structs
//     {
//         let abs_enums = build_abs_enums(
//             &parsed_input,
//             &root_struct_name,
//             &flattened_fields,
//             &inheritence_map,
//         );
//         res.extend(abs_enums);
//     }

    res.into()
}

#[allow(dead_code)]
fn get_crate_path() -> TokenStream2 {
    match crate_name(CRATE_NAME) {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote!(#ident)
        }
        Err(_) => panic!("Could not find the `{}` crate.", CRATE_NAME),
    }
}
