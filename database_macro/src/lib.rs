mod attributes;
mod content_impl;
mod database_impl;
mod derive_database;
mod derive_subset;
mod dromedar_case;
mod enum_impl;
mod subscriber_handler_impl;

use core::panic;
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

const CRATE_NAME: &str = "database";

#[proc_macro_derive(Database, attributes(name, subset))]
pub fn derive_database(input: TokenStream) -> TokenStream {
    derive_database::derive_database(input)
}

#[proc_macro_derive(Subset, attributes(superset))]
pub fn derive_subset(input: TokenStream) -> TokenStream {
    derive_subset::derive_subset(input)
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
