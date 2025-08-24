use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Ident, parse_macro_input};

use crate::dromedar_case::to_dromedar_case;
use crate::{
    attributes::{extract_fields, extract_subset_attributes},
    derive_database::Field,
    dromedar_case::to_upper_snake_case,
    get_crate_path,
};

pub(crate) fn derive_subset(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let crate_path = get_crate_path();
    let name = input.ident.clone();

    let superset = extract_subset_attributes(&input);
    let fields: Vec<Field> = extract_fields(&input);

    let enum_name_str = format!("{}Parameters", superset);
    let enum_name_ident = Ident::new(&enum_name_str, Span::call_site());
    let enum_size_str = format!("{}_COUNT", to_upper_snake_case(&enum_name_str));
    let enum_size_ident = Ident::new(&enum_size_str, Span::call_site());

    let subscriber_name = format!("{}Subscriber", name);
    let subscriber_name_ident = Ident::new(&subscriber_name, Span::call_site());

    // Generate a check for if a specified field has been changed in the parameter change list
    //
    // Expands to the following
    //
    // let alice_index: usize = MyDatabaseParameters::Alice(u8::default()).into();
    // assert!(alice_index < MY_DATABASE_CONTENT_PARAMETERS_COUNT);
    // if parameter_change[alice_zevs_index].is_some() {
    //     parameter_changed = true;
    // }
    let subset_indices: TokenStream2 = fields
        .iter()
        .map(|field| {
            let index_name = Ident::new(
                &format!("{}_index", &field.field_name.to_string()),
                Span::call_site(),
            );

            let variant_name_str = to_dromedar_case(&field.field_name.to_string());
            let variant_ident = Ident::new(&variant_name_str, field.field_name.span());
            let ty = &field.field_type;

            quote! {
                let #index_name: usize = #enum_name_ident::#variant_ident(#ty::default()).into();
                assert!(#index_name < #enum_size_ident);
                if parameter_change[#index_name].is_some() {
                    parameter_changed = true;
                }
            }
        })
        .collect();

    // Generate a get request from a database that either gets the value from a change list or the
    // internal contents.
    //
    // Expands to the following
    //
    // let alice = match database.internal_get(&MyDatabaseParameters::Alice(u8::default())) {
    //     MyDatabaseParameters::Alice(value) => value,
    //     _ => unreachable!(),
    // };
    let field_construction: TokenStream2 = fields
        .iter()
        .map(|field| {
            let field_name = field.field_name;

            let variant_name_str = to_dromedar_case(&field.field_name.to_string());
            let variant_ident = Ident::new(&variant_name_str, field.field_name.span());
            let ty = &field.field_type;

            quote! {
                let #field_name = match database.internal_get(&#enum_name_ident::#variant_ident(#ty::default())) {
                    #enum_name_ident::#variant_ident(value) => value,
                    _ => unreachable!(),
                };
            }
        })
        .collect();

    // Generate a list of comma-seperated fields
    let self_construction: TokenStream2 = fields
        .iter()
        .map(|field| {
            let field_name = field.field_name;
            quote! {
                #field_name,
            }
        })
        .collect();

    let expanded = quote! {
        impl #crate_path::Subset<#enum_name_ident, 3> for #name {
            fn is_subscribed(parameter_change: &#crate_path::ParameterChangeList<#enum_name_ident, #enum_size_ident>) -> bool {
                let mut parameter_changed = false;

                #subset_indices

                parameter_changed
            }

            fn build_from_database(database: &dyn #crate_path::DatabaseRef<#enum_name_ident>) -> Self {

                #field_construction

                Self {
                    #self_construction
                }
            }
        }

        // Type definition of what to implement to get `on_set`
        // pub trait #subscriber_name_ident :#crate_path::DatabaseSubscriber<#name, #superset, #enum_size_ident> {}
    };

    TokenStream::from(expanded)
}
