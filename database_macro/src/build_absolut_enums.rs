use std::any::Any;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::{Data, DeriveInput, Fields, Type};

use crate::{build_parameter_type_lists::ParameterMap, to_dromedar_case};

// Build absolute enums from the form
// ```
//  struct MyMacroFlatData {
//      param1: u8,
//      param2: bool,
//      param3: u8,
//      #[folder]
//      inner1: MyMacroInnerData,
//      #[folder]
//      inner2: MyMacroInnerData,
//  }
// ```
//
// To
//
// ```
//  pub enum MyMacroFlatDataAbsKey {
//      Param1,
//      Param3,
//      Param2,
//      Inner1(MyMacroInnerDataAbsKey),
//      Inner2(MyMacroInnerDataAbsKey),
//  }
//  pub enum MyMacroFlatDataAbsField {
//      Param1(u8),
//      Param3(u8),
//      Param2(bool),
//      Inner1(MyMacroInnerDataAbsField),
//      Inner2(MyMacroInnerDataAbsField),
//  }
// ````
pub(crate) fn build_absolut_enums(
    input: &DeriveInput,
    parameter_map: &ParameterMap,
) -> TokenStream {
    let mut key_variants: Vec<TokenStream2> = Vec::new();
    let mut field_variants: Vec<TokenStream2> = Vec::new();

    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields.named.clone(),
            _ => panic!("Only named fields supported"),
        },
        _ => panic!("Only structs supported"),
    };

    for (_ty_string, entries) in parameter_map {
        if entries.is_folder {
            if let Type::Path(type_path) = &entries.params[0].0 {
                // Build the enum name for the inner type
                let inner_type = &type_path.path.segments.last().unwrap().ident;
                let inner_type_camel_case = to_dromedar_case(&inner_type.to_string());
                let inner_enum_key_name = format_ident!("{}AbsKey", inner_type_camel_case.clone());
                let inner_enum_field_name =
                    format_ident!("{}AbsField", inner_type_camel_case.clone());

                for (_ty, ident) in entries.params.clone() {
                    let inner_name = ident.to_string();
                    let inner_name_camel_case = format_ident!("{}", to_dromedar_case(&inner_name));

                    key_variants.push(quote! {#inner_name_camel_case(#inner_enum_key_name)});
                    field_variants.push(quote! {#inner_name_camel_case(#inner_enum_field_name)});
                }
            }
        } else {
            for (ty, ident) in entries.params.clone() {
                let inner_name = ident.to_string();
                let inner_name_camel_case = format_ident!("{}", to_dromedar_case(&inner_name));

                key_variants.push(quote! {#inner_name_camel_case});
                field_variants.push(quote! {#inner_name_camel_case(#ty)});
            }
        }
    }

    let keys_enum_name = format_ident!("{}AbsKey", input.ident);
    let fields_enum_name = format_ident!("{}AbsField", input.ident);

    let expanded = quote! {
        pub enum #keys_enum_name {
            #(#key_variants),*
        }

        pub enum #fields_enum_name {
            #(#field_variants),*
        }
    };

    // eprintln!("{}", expanded);

    TokenStream::from(expanded)
}
