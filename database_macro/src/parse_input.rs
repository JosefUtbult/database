use syn::{
    ItemStruct, Result, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

pub(crate) struct ParsedInput {
    pub(crate) structs: Vec<ItemStruct>,
}

impl Parse for ParsedInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let structs_punct = Punctuated::<ItemStruct, Token![,]>::parse_terminated(input)?;
        let structs = structs_punct.into_iter().collect();

        Ok(ParsedInput { structs })
    }
}
