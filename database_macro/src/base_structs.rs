use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::ParsedInput;

pub(crate) fn rebuild_structs(parsed_input: &ParsedInput) -> TokenStream2 {
    let mut res = TokenStream2::new();
    for struct_instance in &parsed_input.structs {
        res.extend(quote! {#struct_instance});
    }

    res
}
