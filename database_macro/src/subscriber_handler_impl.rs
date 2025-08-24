use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::Ident;

use crate::dromedar_case::to_snake_case;

pub(crate) fn generate_subscriber_handler_impl(
    crate_path: &TokenStream2,
    struct_name: &Ident,
    enum_name: &Ident,
    enum_size: &Ident,
    subsets: &Vec<Ident>,
) -> TokenStream2 {
    let handler_ident = Ident::new(
        &format!("{}SubscriberHandler", struct_name),
        Span::call_site(),
    );

    // Generates a variable declaration for a subscriber handler struct implementation.
    // Expands to the following
    //
    //  my_subset1_subscribers: [Option<&'a dyn database::DatabaseSubscriber<MySubset1>>; 4],
    let subset_variables: TokenStream2 = subsets
        .iter()
        .map(|subset| {
            let name = Ident::new(
                &format!("{}_subscribers", to_snake_case(&subset.to_string())),
                Span::call_site(),
            );

            quote! {
                #name: [Option<&'a dyn #crate_path::DatabaseSubscriber<#subset, #enum_name, #enum_size>>; 4],
            }
        })
        .collect();

    // Generates a new implementation for a subscribers list. Expands to the following
    //
    // my_subset1_subscribers: [None; 4],
    let subset_new_values: TokenStream2 = subsets
        .iter()
        .map(|subset| {
            let name = Ident::new(
                &format!("{}_subscribers", to_snake_case(&subset.to_string())),
                Span::call_site(),
            );

            quote! {
                #name: [None; 4],
            }
        })
        .collect();

    // Generate a subscribe function. Expands to the following
    //
    // pub fn subscribe_with_my_subset1(
    //     &mut self,
    //     subscriber: &'a dyn database::DatabaseSubscriber<MySubset1>,
    // ) -> Result<(), database::DatabaseError> {
    //     for instance in self.my_subset1_subscribers.iter_mut() {
    //         if instance.is_none() {
    //             let _ = instance.insert(subscriber);
    //             return Ok(());
    //         }
    //     }
    //     Err(DatabaseError::SubscriberOverflow)
    // }
    let subset_subscribe_function: TokenStream2 = subsets
        .iter()
        .map(|subset| {
            let variable_name = Ident::new(
                &format!("{}_subscribers", to_snake_case(&subset.to_string())),
                Span::call_site(),
            );

            let function_name = Ident::new(
                &format!("subscribe_with_{}", to_snake_case(&subset.to_string())),
                Span::call_site(),
            );

            quote! {
                pub fn #function_name(
                    &mut self,
                    subscriber: &'a dyn #crate_path::DatabaseSubscriber<#subset, #enum_name, #enum_size>
                ) -> Result<(), #crate_path::DatabaseError> {
                    for instance in self.#variable_name.iter_mut() {
                        if instance.is_none() {
                            let _ = instance.insert(subscriber);
                            return Ok(());
                        }
                    }
                    Err(#crate_path::DatabaseError::SubscriberOverflow)
                }
            }
        })
        .collect();

    // Generates a notify implementation for a specific subset. Utilizes in-built functionality in
    // the subset trait to check if a subset is subscribed to a specific parameter, and notifies
    // all relevant subscribers if a parameter has changed
    //
    // Expands to the following
    //
    // {
    //     if MySubset1::is_subscribed(parameter_change) {
    //         let subset = MySubset1::build_from_database(database);
    //         for instance in self.my_subset1_subscribers.iter() {
    //             if let Some(instance) = instance {
    //                 instance.on_set(&subset);
    //             }
    //         }
    //     }
    // }
    let subset_notify: TokenStream2 = subsets
        .iter()
        .map(|subset| {
            let variable_name = Ident::new(
                &format!("{}_subscribers", to_snake_case(&subset.to_string())),
                Span::call_site(),
            );

            quote! {
                {
                    if #subset::is_subscribed(parameter_change) {
                        let subset = #subset::build_from_database(database);
                        for instance in self.#variable_name.iter() {
                            if let Some(instance) = instance {
                                instance.on_set(&subset);
                            }
                        }
                    }
                }

            }
        })
        .collect();

    // Struct declaration and implementation for a database subscriber handler. Implements the
    // `DatabaseSubscriberHandler` trait, and expands `notify_subscribers` to go through each
    // registered subset
    quote! {
        pub struct #handler_ident<'a> {
            #subset_variables
        }

        #[automatically_derived]
        impl<'a> #handler_ident<'a> {
            pub const fn new() -> Self {
                Self {
                    #subset_new_values
                }
            }

            #subset_subscribe_function
        }

        #[automatically_derived]
        impl<'a> #crate_path::DatabaseSubscriberHandler<'a, #struct_name, #enum_name, #enum_size>
            for #handler_ident<'a>
        {
            fn notify_subscribers(
                &self,
                database: &dyn #crate_path::DatabaseRef<#enum_name>,
                parameter_change: &#crate_path::ParameterChangeList<#enum_name, #enum_size>,
            ) {
                use #crate_path::Subset;

                #subset_notify
            }
        }
    }
}
