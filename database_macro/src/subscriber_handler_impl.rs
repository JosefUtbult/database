use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::Ident;

use crate::{Field, to_dromedar_case};

pub(crate) fn generate_subscriber_handler_impl(
    crate_path: &TokenStream2,
    struct_name: &Ident,
    enum_name: &Ident,
    enum_size: &Ident,
    fields: &[Field],
) -> TokenStream2 {
    let handler_ident = Ident::new(
        &format!("{}SubscriberHandler", struct_name),
        Span::call_site(),
    );

    // Extract match arms for every field
    let field_matches = fields.iter().map(|field| {
        let field_ident = &field.field_name;
        let variant_name_str = to_dromedar_case(&field_ident.to_string());
        let variant_ident = Ident::new(&variant_name_str, field_ident.span());
        let field_ty = &field.field_type;

        quote! {
            #[allow(unused_variables)]
            let #field_ident = match database.internal_get(
                &#enum_name::#variant_ident(<#field_ty>::default())
            ) {
                #enum_name::#variant_ident(value) => value,
                _ => unreachable!(),
            };
        }
    });

    quote! {
        pub struct #handler_ident<'a> {
            my_content_subset_subscribers: [Option<&'a dyn DatabaseSubscriber<MyContentSubset>>; 128],
        }

        impl<'a> #handler_ident<'a> {
            pub const fn new() -> Self {
                Self {
                    my_content_subset_subscribers: [None; 128],
                }
            }

            pub fn subscribe_with_content_subset(
                &mut self,
                subscriber: &'a dyn DatabaseSubscriber<MyContentSubset>,
            ) -> Result<(), DatabaseError> {
                for instance in self.my_content_subset_subscribers.iter_mut() {
                    if instance.is_none() {
                        let _ = instance.insert(subscriber);
                        return Ok(());
                    }
                }
                Err(DatabaseError::SubscriberOverflow)
            }
        }

        impl<'a> DatabaseSubscriberHandler<#struct_name, #enum_name, #enum_size>
            for #handler_ident<'a>
        {
            fn notify_subscribers(
                &self,
                database: &dyn #crate_path::DatabaseRef<#enum_name>,
                parameter_change: &#crate_path::ParameterChangeList<#enum_name, #enum_size>,
            ) {
                #(#field_matches)*

                // Here you could add subset notification logic dynamically
            }
        }
    }
}
