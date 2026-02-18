use proc_macro2::Ident;
use syn::{
    parse::{Parse, ParseStream}, punctuated::Punctuated, ItemStruct, Result, Token
};

pub(crate) struct ParsedInput {
    pub(crate) name: Ident,
    pub(crate) structs: Vec<ItemStruct>,
}

impl Parse for ParsedInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        let structs_punct = Punctuated::<ItemStruct, Token![,]>::parse_terminated(input)?;
        let structs = structs_punct.into_iter().collect();

        Ok(ParsedInput { name, structs })
    }
}
