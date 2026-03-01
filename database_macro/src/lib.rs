mod casing;
mod data_field_accessor;
mod data_structure;
mod database;
mod enums;
mod focus_handler;
mod parse_input;
mod structs;

use syn::parse_macro_input;

use core::panic;
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::{
    data_field_accessor::generate_data_field_accessors::generate_data_field_accessors,
    data_structure::{DataStructure, build_data_structure},
    database::generate_database::generate_database,
    enums::{
        generate_abs_enums, generate_abs_from, generate_abs_impl_debug,
        generate_database_constraints, generate_flat_enums, generate_flat_from,
        generate_flat_impl_debug,
    },
    focus_handler::generate_focus_handler,
    parse_input::ParsedInput,
    structs::re_add_structs::re_add_structs,
};

const CRATE_NAME: &str = "database";

#[proc_macro]
pub fn build_database(input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as ParsedInput);

    let mut res = TokenStream2::new();
    {
        let stream = re_add_structs(&parsed_input);
        res.extend(stream);
    }

    let data_structure = build_data_structure(parsed_input);

    let crate_path = get_crate_path();
    for struct_name in data_structure.struct_names.iter() {
        let struct_data = data_structure.struct_map.get(struct_name).unwrap();
        {
            let stream = generate_abs_enums(&crate_path, &data_structure, &struct_data);
            res.extend(stream);
        }

        {
            let stream = generate_abs_from(&crate_path, &data_structure, &struct_data);
            res.extend(stream);
        }

        {
            let stream = generate_abs_impl_debug(&crate_path, &data_structure, &struct_data);
            res.extend(stream);
        }

        {
            let stream = generate_data_field_accessors(&crate_path, &data_structure, &struct_data);
            res.extend(stream);
        }
    }

    {
        let stream = generate_flat_enums(&crate_path, &data_structure);
        res.extend(stream);
    }

    {
        let stream = generate_flat_impl_debug(&crate_path, &data_structure);
        res.extend(stream);
    }

    {
        let stream = generate_database_constraints(&crate_path, &data_structure);
        res.extend(stream);
    }

    {
        let stream = generate_flat_from(&crate_path, &data_structure);
        res.extend(stream);
    }

    {
        let stream = generate_focus_handler(&crate_path, &data_structure);
        res.extend(stream);
    }

    {
        let stream = generate_database(&crate_path, &data_structure);
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
