use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Ident;

pub(crate) fn generate_database_impl(
    crate_path: &TokenStream2,
    database_name: &Ident,
    struct_name: &Ident,
    enum_name: &Ident,
    enum_size: &Ident,
) -> TokenStream2 {
    let subscriber_handler_ident = Ident::new(
        &format!("{}SubscriberHandler", struct_name),
        Span::call_site(),
    );

    // Build full impl
    quote! {
        pub struct #database_name<'a>(#crate_path::DatabaseHandler<'a, #struct_name, #subscriber_handler_ident<'a>, #enum_name, #enum_size>);

        impl<'a> #database_name<'a> {
            pub fn new(content: #struct_name) -> Self {
                Self(#crate_path::DatabaseHandler::new(content, #subscriber_handler_ident::new()))
            }

            /// Retrieve a value from the database
            pub fn get(&self, parameter: &#enum_name) -> #enum_name {
                self.0.get(parameter)
            }

            /// Set an array of parameters in a database. This will store a changed state for the provided
            /// parameters, which later is acted upon by calling the `notify_subscribers` function
            pub fn multi_set(&self, parameters: &[#enum_name]) {
                self.0.multi_set(parameters)
            }

            /// Set a parameter in a database. This will store a changed state for the provided
            /// parameter, which later is acted upon by calling the `notify_subscribers` function
            pub fn set(&self, parameter: &#enum_name) {
                self.0.set(parameter)
            }

            /// Notify all subscribers of changes made to the database. This is separated out from the set
            /// functionality, as these might need to run under different contexts/priority levels. This
            /// function presumes that no other entity is actively handling the list of internal
            /// subscribers. If the internal subscribers are locked for any reason, this will cause a
            /// `DatabaseError`
            pub fn notify_subscribers(&self) -> Result<(), #crate_path::DatabaseError> {
                self.0.notify_subscribers()
            }

            /// Retrieve a handle to the internal subscriber handler. Used to subscribe to different
            /// subsets of the parameter space. This should be done before actively using the database, as
            /// this can cause locking errors resulting in a failure to notify subscribers
            pub fn get_subscriber_handler(&'a self) -> &'a #crate_path::SpinMutex<#crate_path::RefCell<#subscriber_handler_ident>> {
                self.0.get_subscriber_handler()
            }
        }
    }
}
