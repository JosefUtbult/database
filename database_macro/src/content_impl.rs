use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Ident;

use crate::{derive_database::Field, dromedar_case::to_dromedar_case};

pub(crate) fn generate_database_content_impl(
    crate_path: &TokenStream2,
    struct_name: &Ident,
    enum_name: &Ident,
    enum_size: &Ident,
    fields: &[Field],
) -> TokenStream2 {
    // Generate match arms for `set`
    let set_arms = fields.iter().map(|field| {
        let variant_name_str = to_dromedar_case(&field.field_name.to_string());
        let variant_ident = Ident::new(&variant_name_str, field.field_name.span());
        let field_name = &field.field_name;

        quote! {
            #enum_name::#variant_ident(value) => self.#field_name = value,
        }
    });

    // Generate match arms for `get`
    let get_arms = fields.iter().map(|field| {
        let variant_name_str = to_dromedar_case(&field.field_name.to_string());
        let variant_ident = Ident::new(&variant_name_str, field.field_name.span());
        let field_name = &field.field_name;

        quote! {
            #enum_name::#variant_ident(_) => #enum_name::#variant_ident(self.#field_name),
        }
    });

    // Build full impl
    quote! {
        impl #crate_path::DatabaseContent<#enum_name, #enum_size> for #struct_name {
            fn set(&mut self, parameter: #enum_name) {
                match parameter {
                    #(#set_arms)*
                }
            }

            fn get(&self, parameter: &#enum_name) -> #enum_name {
                match parameter {
                    #(#get_arms)*
                }
            }
        }
    }
}
