use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, Type, parse_macro_input};

struct Field<'a> {
    field_name: &'a Ident,
    field_type: &'a Type,
}

const CRATE_NAME: &str = "database";

#[proc_macro_derive(Database)]
pub fn derive_database(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let crate_path = get_crate_path();
    let name = input.ident.clone();
    let fields = extract_fields(&input);

    let enum_name_str = format!("{}Parameters", name);
    let enum_ident = Ident::new(&enum_name_str, Span::call_site());

    let parameters_enum = generate_parameters_enum(&name, &enum_ident, &fields);
    let content_implementation =
        generate_database_content_impl(&crate_path, &name, &enum_ident, &fields);

    let expanded = quote! {
        #parameters_enum

        #content_implementation
    };

    TokenStream::from(expanded)
}

fn get_crate_path() -> TokenStream2 {
    match crate_name(CRATE_NAME) {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote!(#ident)
        }
        Err(_) => panic!("Could not find the `{}` crate.", CRATE_NAME),
    }
}

fn extract_fields(input: &DeriveInput) -> Vec<Field> {
    // Extract the fields from the input
    let fields = match input.data {
        Data::Struct(ref data_struct) => &data_struct.fields,
        _ => panic!("#[derive(Database)] can only be applied to structs"),
    };

    // Collect field names and types
    let mut field_info = Vec::new();
    if let Fields::Named(named_fields) = fields {
        for field in &named_fields.named {
            let field_name = field.ident.as_ref().unwrap();
            let field_type = &field.ty;
            field_info.push(Field {
                field_name,
                field_type,
            });
        }
    }

    field_info
}

fn to_dromedar_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

fn generate_parameters_enum(name: &Ident, enum_ident: &Ident, fields: &[Field]) -> TokenStream2 {
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
            quote! { #enum_ident::#variant(_) => #idx, }
        })
        .collect();

    // Combine enum + From impl
    quote! {
        #[allow(dead_code)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum #enum_ident {
            #variants_tokens
        }

        impl From<#enum_ident> for usize {
            fn from(value: #enum_ident) -> Self {
                match value {
                    #from_arms
                }
            }
        }
    }
}

fn generate_database_content_impl(
    crate_path: &TokenStream2,
    struct_name: &Ident,
    enum_ident: &Ident,
    fields: &[Field],
) -> TokenStream2 {
    let parameter_count = fields.len();

    // Generate match arms for `set`
    let set_arms = fields.iter().map(|field| {
        let variant_name_str = to_dromedar_case(&field.field_name.to_string());
        let variant_ident = Ident::new(&variant_name_str, field.field_name.span());
        let field_name = &field.field_name;

        quote! {
            #enum_ident::#variant_ident(value) => self.#field_name = value,
        }
    });

    // Generate match arms for `get`
    let get_arms = fields.iter().map(|field| {
        let variant_name_str = to_dromedar_case(&field.field_name.to_string());
        let variant_ident = Ident::new(&variant_name_str, field.field_name.span());
        let field_name = &field.field_name;

        quote! {
            #enum_ident::#variant_ident(_) => #enum_ident::#variant_ident(self.#field_name),
        }
    });

    // Build full impl
    quote! {
        impl #crate_path::DatabaseContent<#enum_ident, #parameter_count> for #struct_name {
            fn set(&mut self, parameter: #enum_ident) {
                match parameter {
                    #(#set_arms)*
                }
            }

            fn get(&self, parameter: &#enum_ident) -> #enum_ident {
                match parameter {
                    #(#get_arms)*
                }
            }
        }
    }
}
