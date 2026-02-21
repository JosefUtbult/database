mod casing;
mod data_structure;
mod enums;
mod parse_input;

use syn::parse_macro_input;

use core::panic;
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::{
    data_structure::{DataStructure, build_data_structure},
    enums::{generate_abs_enums, generate_abs_from, generate_flat_enums, generate_flat_from},
    parse_input::ParsedInput,
};

const CRATE_NAME: &str = "database";

#[proc_macro]
pub fn build_database(input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as ParsedInput);
    let data_structure = build_data_structure(parsed_input);

    let crate_path = get_crate_path();
    let mut res = TokenStream2::new();
    for (_, struct_data) in data_structure.struct_map.iter() {
        {
            let stream = generate_abs_enums(&crate_path, &data_structure, &struct_data);
            res.extend(stream);
        }

        {
            let stream = generate_abs_from(&crate_path, &data_structure, &struct_data);
            res.extend(stream);
        }
    }

    {
        let stream = generate_flat_enums(&crate_path, &data_structure);
        res.extend(stream);
    }

    {
        let stream = generate_flat_from(&crate_path, &data_structure);
        res.extend(stream);
    }

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
