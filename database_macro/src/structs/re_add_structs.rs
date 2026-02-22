use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::ParsedInput;

pub(crate) fn re_add_structs(parsed_input: &ParsedInput) -> TokenStream2 {
    let original_structs = parsed_input.structs.iter();

    quote! {
        #(#original_structs)*
    }
}
