mod attributes;
mod content_impl;
mod dromedar_case;
mod enum_impl;
mod subscriber_handler_impl;

use core::panic;
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Type};

use crate::{
    attributes::{extract_attributes, extract_fields}, content_impl::generate_database_content_impl, dromedar_case::{to_dromedar_case, to_upper_snake_case}, enum_impl::generate_parameters_enum, subscriber_handler_impl::generate_subscriber_handler_impl
};

struct Field<'a> {
    field_name: &'a Ident,
    field_type: &'a Type,
}

const CRATE_NAME: &str = "database";

#[proc_macro_derive(Database, attributes(name, subset))]
pub fn derive_database(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let crate_path = get_crate_path();
    let name = input.ident.clone();
    let (database_name, subsets) = extract_attributes(&input);
    let fields = extract_fields(&input);

    let enum_name_str = format!("{}Parameters", name);
    let enum_name_ident = Ident::new(&enum_name_str, Span::call_site());
    let enum_size_str = format!("{}_COUNT", to_upper_snake_case(&enum_name_str));
    let enum_size_ident = Ident::new(&enum_size_str, Span::call_site());

    let parameters_enum = generate_parameters_enum(&enum_name_ident, &enum_size_ident, &fields);

    let content_implementation = generate_database_content_impl(
        &crate_path,
        &name,
        &enum_name_ident,
        &enum_size_ident,
        &fields,
    );

    let subscriber_handler_impl = generate_subscriber_handler_impl(
        &crate_path,
        &name,
        &enum_name_ident,
        &enum_size_ident,
        &fields,
    );

    let expanded = quote! {
        // #parameters_enum

        // #content_implementation

        #subscriber_handler_impl
    };

    TokenStream::from(expanded)
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
