use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, Type, parse_macro_input};

use crate::{
    attributes::{extract_database_attributes, extract_fields},
    content_impl::generate_database_content_impl,
    database_impl::generate_database_impl,
    enum_impl::{generate_enum_names, generate_parameters_enum},
    get_crate_path,
    subscriber_handler_impl::generate_subscriber_handler_impl,
};

pub(crate) struct Field<'a> {
    pub(crate) field_name: &'a Ident,
    pub(crate) field_type: &'a Type,
}

pub(crate) fn derive_database(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let crate_path = get_crate_path();
    let name = input.ident.clone();
    let (database_name, subsets) = extract_database_attributes(&input);
    let fields: Vec<Field> = extract_fields(&input);

    let (enum_name_ident, enum_size_ident) = generate_enum_names(&database_name);

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
        &subsets,
    );

    let database_impl = generate_database_impl(
        &crate_path,
        &database_name,
        &name,
        &enum_name_ident,
        &enum_size_ident,
    );

    let expanded = quote! {
        #parameters_enum

        #content_implementation

        #subscriber_handler_impl

        #database_impl
    };

    TokenStream::from(expanded)
}
