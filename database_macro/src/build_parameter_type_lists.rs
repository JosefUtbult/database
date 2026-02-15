use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

pub(crate) struct TypeInfo {
    pub(crate) is_folder: bool,
    pub(crate) params: Vec<(syn::Type, syn::Ident)>,
}

pub(crate) type ParameterMap = HashMap<String, TypeInfo>;

pub(crate) fn build_parameter_type_lists(input: &DeriveInput) -> (ParameterMap, bool) {
    let mut type_map = ParameterMap::new();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Only named fields supported"),
        },
        _ => panic!("Only structs supported"),
    };

    let mut has_folders = false;
    for field in fields.iter() {
        let field = field.clone();
        let ty = field.ty;
        let ident = field.ident.unwrap();

        // Check if folder
        let is_folder = field
            .attrs
            .iter()
            .any(|attr| {
                attr.path.is_ident("folder")
            });

        let key = quote!(#ty).to_string();
        let type_info = type_map.entry(key).or_insert(TypeInfo {
            is_folder: false,
            params: Vec::new(),
        });

        type_info.params.push((ty.clone(), ident));

        if is_folder {
            has_folders = true;
            type_info.is_folder = true;
        }
    }

    (type_map, has_folders)
}
