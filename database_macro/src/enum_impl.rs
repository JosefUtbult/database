use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Ident;

use crate::{
    derive_database::Field,
    dromedar_case::{to_dromedar_case, to_upper_snake_case},
};

pub(crate) fn generate_enum_names(database_name: &Ident) -> (Ident, Ident) {
    let enum_name_str = format!("{}Member", database_name);
    let enum_name_ident = Ident::new(&enum_name_str, Span::call_site());
    let enum_size_str = format!("{}_COUNT", to_upper_snake_case(&enum_name_str));
    let enum_size_ident = Ident::new(&enum_size_str, Span::call_site());

    (enum_name_ident, enum_size_ident)
}

pub(crate) fn generate_parameters_enum(
    enum_name: &Ident,
    enum_size: &Ident,
    fields: &[Field],
) -> TokenStream2 {
    // Generate enum variants of struct members
    let mut variant_idents = Vec::new();
    let variants_tokens: TokenStream2 = fields
        .iter()
        .map(|field| {
            let variant_name_str = to_dromedar_case(&field.field_name.to_string());
            let variant_ident = Ident::new(&variant_name_str, field.field_name.span());
            variant_idents.push(variant_ident.clone());

            let ty = &field.field_type;
            quote! { #variant_ident(#ty), }
        })
        .collect();

    // Generate From<Enum> for usize implementation
    let from_arms: TokenStream2 = variant_idents
        .iter()
        .enumerate()
        .map(|(idx, variant)| {
            quote! { #enum_name::#variant(_) => #idx, }
        })
        .collect();

    let param_count = fields.len();

    // Combine enum + From impl
    quote! {
        pub const #enum_size: usize = #param_count;

        #[allow(dead_code)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum #enum_name {
            #variants_tokens
        }

        impl From<#enum_name> for usize {
            fn from(value: #enum_name) -> Self {
                match value {
                    #from_arms
                }
            }
        }
    }
}
