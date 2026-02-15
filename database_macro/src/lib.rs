mod parse_input;

mod build_absolut_enums;
mod build_parameter_type_lists;

mod build_flat_enums;
use build_flat_enums::*;

mod casing;
use casing::*;
use syn::parse_macro_input;

mod base_structs;

use core::panic;
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::{base_structs::rebuild_structs, parse_input::ParsedInput};

const CRATE_NAME: &str = "database";

#[proc_macro]
pub fn build_database(input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as ParsedInput);
    let mut res = TokenStream2::new();

    let inheritence_map = map_struct_inheritence(&parsed_input.structs);
    eprintln!("Inheritence: {:?}", inheritence_map);
    panic!("");

    // Add back the input structs to the output stream
    {
        let base_structs = rebuild_structs(&parsed_input);
        res.extend(base_structs);
    }

    // Build flat enums for all database fields
    {
        let flat_enums = build_flat_enums(&parsed_input);
        res.extend(flat_enums);
    }

    res.into()
}

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
