use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};

use crate::DataStructure;

fn generate_folder_enums(_crate_path: &TokenStream2, _data_structure: &DataStructure) -> TokenStream2 {
    quote! {}
}

fn generate_focus_enums(_crate_path: &TokenStream2, data_structure: &DataStructure) -> TokenStream2 {
    let mut result = TokenStream2::new();

    for (_, folder_info) in data_structure.all_folder_fields.iter() {
        if folder_info.fields.len() <= 1 {
            continue;
        }

        let folder_key_enum = folder_info.folder_enum_name.clone();

        let mut folder_key_variants: Vec<Ident> = Vec::new();
        let mut from_usize_to_variant: Vec<TokenStream2> = Vec::new();
        let mut from_variant_to_usize: Vec<TokenStream2> = Vec::new();

        for (index, field_info) in folder_info.fields.iter().enumerate() {
            let index = index as u8;

            let variant_name = field_info.variant_name.clone();
            folder_key_variants.push(variant_name.clone());

            from_usize_to_variant.push(quote! {
                #index => Ok(#folder_key_enum::#variant_name)
            });

            from_variant_to_usize.push(quote! {
                #folder_key_enum::#variant_name => #index
            });
        }

        // #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
        result.extend(quote! {
            pub enum #folder_key_enum {
                #(#folder_key_variants,)*
            }

            #[automatically_derived]
            impl From<#folder_key_enum> for u8 {
                fn from(value: #folder_key_enum) -> Self {
                    match value {
                        #(#from_variant_to_usize,)*
                    }
                }
            }

            #[automatically_derived]
            impl TryFrom<u8> for #folder_key_enum {
                type Error = ();

                fn try_from(value: u8) -> Result<Self, Self::Error> {
                    match value {
                        #(#from_usize_to_variant,)*
                        _ => Err(())
                    }
                }
            }
        });
    }

    result
}

fn generate_handler_declaration(
    crate_path: &TokenStream2,
    data_structure: &DataStructure,
) -> TokenStream2 {
    let root_struct = data_structure.root_struct.as_ref().unwrap();
    let focus_handler_name = data_structure
        .type_names
        .as_ref()
        .unwrap()
        .focus_handler_name
        .clone();

    let abs_key_enum = root_struct.type_names.abs_key_enum.clone();
    let abs_field_enum = root_struct.type_names.abs_field_enum.clone();

    let flat_key_enum = root_struct.type_names.flat_key_enum.clone();
    let flat_field_enum = root_struct.type_names.flat_field_enum.clone();

    let mut folder_key_declarations: Vec<TokenStream2> = Vec::new();

    for (_, folder_info) in data_structure.all_folder_fields.iter() {
        if folder_info.fields.len() <= 1 {
            continue;
        }

        let folder_key_enum = folder_info.folder_enum_name.clone();
        let folder_focus_variable = folder_info.folder_focus_variable.clone();

        assert!(
            folder_info.fields.len() < u8::MAX as usize,
            "Folder {:?} exists in to many variants ({:?})",
            folder_key_enum.to_string(),
            folder_info.fields.len()
        );

        folder_key_declarations.push(quote! {
            #folder_focus_variable: core::sync::atomic::AtomicU8
        });
    }

    quote! {
        #[automatically_derived]
        pub struct #focus_handler_name {
            #(#folder_key_declarations,)*
        }

        #[automatically_derived]
        impl #crate_path::FocusHandler<#abs_key_enum, #abs_field_enum, #flat_key_enum, #flat_field_enum> for #focus_handler_name {
            fn get_focus_key(&self, key: #flat_key_enum) -> #abs_key_enum {
                todo!();
            }

            fn get_focus_field(&self, field: #flat_field_enum) -> #abs_field_enum {
                todo!();
            }
        }
    }
}

fn generate_new(_crate_path: &TokenStream2, data_structure: &DataStructure) -> TokenStream2 {
    let mut folder_key_definitions: Vec<TokenStream2> = Vec::new();
    for (_, folder_info) in data_structure.all_folder_fields.iter() {
        if folder_info.fields.len() <= 1 {
            continue;
        }

        let folder_key_enum = folder_info.folder_enum_name.clone();
        let folder_focus_variable = folder_info.folder_focus_variable.clone();

        assert!(
            folder_info.fields.len() < u8::MAX as usize,
            "Folder {:?} exists in to many variants ({:?})",
            folder_key_enum.to_string(),
            folder_info.fields.len()
        );

        folder_key_definitions.push(quote! {
            #folder_focus_variable: core::sync::atomic::AtomicU8::new(0)
        });
    }

    quote! {
        pub const fn new() -> Self {
            Self {
                #(#folder_key_definitions,)*
            }
        }
    }
}

fn generate_get_set(_crate_path: &TokenStream2, data_structure: &DataStructure) -> TokenStream2 {
    let mut get_set_focused_key_functions: Vec<TokenStream2> = Vec::new();

    for (_, folder_info) in data_structure.all_folder_fields.iter() {
        if folder_info.fields.len() <= 1 {
            continue;
        }

        let folder_key_enum = folder_info.folder_enum_name.clone();
        let folder_focus_variable = folder_info.folder_focus_variable.clone();
        let get_function_name = format_ident!("get_{}", folder_focus_variable);
        let set_function_name = format_ident!("set_{}", folder_focus_variable);

        get_set_focused_key_functions.push(quote! {
            pub fn #get_function_name(&self) -> #folder_key_enum {
                self.#folder_focus_variable.load(core::sync::atomic::Ordering::SeqCst).try_into().unwrap()
            }
            pub fn #set_function_name(&self, focus: #folder_key_enum) {
                self.#folder_focus_variable.store(focus.into(), core::sync::atomic::Ordering::SeqCst)
            }
        })
    }

    quote! {
        #(#get_set_focused_key_functions)*
    }
}

pub(crate) fn generate_handler_impl(
    crate_path: &TokenStream2,
    data_structure: &DataStructure,
) -> TokenStream2 {
    let focus_handler_name = data_structure
        .type_names
        .as_ref()
        .unwrap()
        .focus_handler_name
        .clone();

    let new_stream = generate_new(crate_path, data_structure);
    let get_set_stream = generate_get_set(crate_path, data_structure);
    quote! {
        #[automatically_derived]
        impl #focus_handler_name {
            #new_stream
            #get_set_stream
        }
    }
}

pub(crate) fn generate_focus_handler(
    crate_path: &TokenStream2,
    data_structure: &DataStructure,
) -> TokenStream2 {
    let mut result = TokenStream2::new();
    result.extend(generate_focus_enums(crate_path, data_structure));
    result.extend(generate_folder_enums(crate_path, data_structure));
    result.extend(generate_handler_declaration(crate_path, data_structure));
    result.extend(generate_handler_impl(crate_path, data_structure));
    result
}
