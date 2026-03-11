use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};

use crate::{
    DataStructure,
    casing::to_camel_case,
    data_structure::{
        self,
        absolute_path::{AbsolutePathField, get_parent_struct_type},
        conditional_paths::{AllFolderFields, FolderFocusPath, FolderFocusPathVector},
    },
    enums::abs_path_to_token_stream,
};

fn generate_folder_enums(
    _crate_path: &TokenStream2,
    _data_structure: &DataStructure,
) -> TokenStream2 {
    quote! {}
}

fn generate_focus_enums(
    _crate_path: &TokenStream2,
    data_structure: &DataStructure,
) -> TokenStream2 {
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
    _crate_path: &TokenStream2,
    data_structure: &DataStructure,
) -> TokenStream2 {
    let focus_handler_name = data_structure
        .type_names
        .as_ref()
        .unwrap()
        .focus_handler_name
        .clone();

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

fn recurse_resulting_path(
    all_folder_fields: &AllFolderFields,
    root_abs_folder_enum: &Ident,
    resulting_path: &[(String, String)],
) -> TokenStream2 {
    let (field_name, parent_folder_name) = resulting_path.last().unwrap();

    eprintln!("Recursing resulting path {:?}", resulting_path);

    let parent_abs_folder_enum =
        if let Some(parent_folder_info) = all_folder_fields.get(parent_folder_name) {
            &parent_folder_info.abs_folder_key_enum
        } else {
            root_abs_folder_enum
        };

    if resulting_path.len() > 1 {
        let field_name = format_ident!("In{}", to_camel_case(field_name));
        let child_path = recurse_resulting_path(
            all_folder_fields,
            root_abs_folder_enum,
            &resulting_path[..resulting_path.len() - 1],
        );
        quote! {
            #parent_abs_folder_enum::#field_name(#child_path)
        }
    } else {
        let field_name = format_ident!("{}", to_camel_case(field_name));
        quote! {
            #parent_abs_folder_enum::#field_name
        }
    }
}

fn recurse_conditional(
    folder_focus_path_vector: &FolderFocusPathVector,
    all_folder_fields: &AllFolderFields,
    root_abs_folder_enum: &Ident,
    resulting_path: &[(String, String)],
) -> TokenStream2 {
    eprintln!(
        "Recursion with {:?}, {:?}",
        folder_focus_path_vector, resulting_path
    );

    if let Some(first) = folder_focus_path_vector.first() {
        let struct_name = &first.struct_name;
        eprintln!(
            "Struct {:?}, focus vector: {:?}",
            struct_name, folder_focus_path_vector
        );

        let build_sub_path = |field: &String, focus_path: &Box<FolderFocusPath>| {
            let mut sub_path = resulting_path.to_vec();
            sub_path.push((field.clone(), focus_path.parent_name.clone()));

            recurse_conditional(
                &focus_path.sub_paths,
                all_folder_fields,
                root_abs_folder_enum,
                &sub_path,
            )
        };

        let is_focus_path_geq_1 = folder_focus_path_vector.len() > 1;
        let is_parent_fields_geq_1 = first.parent_fields.len() > 1;
        if is_focus_path_geq_1 || is_parent_fields_geq_1 {
            let folder_info = all_folder_fields.get(struct_name).unwrap();
            let folder_focus_variable = folder_info.folder_focus_variable.clone();
            let get_function_name = format_ident!("get_{}", folder_focus_variable);
            let folder_key_enum = folder_info.folder_enum_name.clone();

            let mut match_streams: Vec<TokenStream2> = Vec::new();
            for focus_path in folder_focus_path_vector.iter() {
                for field in focus_path.parent_fields.iter() {
                    let field_name = format_ident!("{}", to_camel_case(field));
                    let sub_stream = build_sub_path(field, focus_path);
                    match_streams.push(quote! {
                        #folder_key_enum::#field_name => #sub_stream
                    });
                }
            }

            quote! {
                match self.#get_function_name() {
                    #(#match_streams,)*
                }
            }
        } else {
            // There is only one path with one field. Continue the recursion into that path
            let first_field = first.parent_fields.first().unwrap();
            eprintln!("Building sub path with {:?}", first_field);
            build_sub_path(first_field, first)
        }
    } else {
        recurse_resulting_path(all_folder_fields, root_abs_folder_enum, resulting_path)
    }
}

fn generate_get_set(_crate_path: &TokenStream2, data_structure: &DataStructure) -> TokenStream2 {
    let mut get_set_focused_key_functions: Vec<TokenStream2> = Vec::new();
    let root_struct = data_structure.root_struct.as_ref().unwrap();
    let abs_folder_enum = root_struct.type_names.abs_folder_enum.clone();

    for (folder_type, folder_info) in data_structure.all_folder_fields.iter() {
        if folder_info.fields.len() <= 1 {
            continue;
        }

        let folder_key_enum = folder_info.folder_enum_name.clone();
        let folder_focus_variable = folder_info.folder_focus_variable.clone();

        let get_function_name = format_ident!("get_{}", folder_focus_variable);
        let set_function_name = format_ident!("set_{}", folder_focus_variable);

        let get_path_function_name = format_ident!("get_{}_path", folder_focus_variable);

        let folder_focus_path_vector = data_structure
            .folder_focus_path_map
            .get(folder_type)
            .unwrap();

        let folder_focus_stream = recurse_conditional(
            folder_focus_path_vector,
            &data_structure.all_folder_fields,
            &abs_folder_enum,
            &Vec::new(),
        );

        get_set_focused_key_functions.push(quote! {
            pub fn #get_function_name(&self) -> #folder_key_enum {
                self.#folder_focus_variable.load(core::sync::atomic::Ordering::SeqCst).try_into().unwrap()
            }
            pub fn #set_function_name(&self, focus: #folder_key_enum) {
                self.#folder_focus_variable.store(focus.into(), core::sync::atomic::Ordering::SeqCst)
            }

            fn #get_path_function_name(&self) -> #abs_folder_enum {
                #folder_focus_stream
            }
        })
    }

    quote! {
        #(#get_set_focused_key_functions)*
    }
}

fn generate_trait_impl(crate_path: &TokenStream2, data_structure: &DataStructure) -> TokenStream2 {
    let root_struct = data_structure.root_struct.as_ref().unwrap();
    let focus_handler_name = data_structure
        .type_names
        .as_ref()
        .unwrap()
        .focus_handler_name
        .clone();

    let root_struct_description = format_ident!("{}Description", &root_struct.name);

    let abs_key_enum = root_struct.type_names.abs_key_enum.clone();
    let abs_field_enum = root_struct.type_names.abs_field_enum.clone();

    let flat_key_enum = root_struct.type_names.flat_key_enum.clone();
    let flat_field_enum = root_struct.type_names.flat_field_enum.clone();

    let mut key_matches: Vec<TokenStream2> = Vec::new();
    let mut _field_matches: Vec<TokenStream2> = Vec::new();

    for (folder_type, field_pair_vector) in root_struct.folder_to_field_map.iter() {
        let folder_info = data_structure.all_folder_fields.get(folder_type).unwrap();

        if folder_info.fields.len() <= 1 {
            for field_pair in field_pair_vector.iter() {
                let (_, abs_path_vector) = root_struct
                    .field_to_field_abs_path_map
                    .iter()
                    .find(|(field_name, _)| *field_name == field_pair.top_field)
                    .unwrap();

                assert!(abs_path_vector.len() == 1);

                let field_name = format_ident!("{}", to_camel_case(&field_pair.top_field));
                let (abs_path_stream, _) = abs_path_to_token_stream(
                    abs_path_vector.first().unwrap(),
                    &None,
                    &abs_key_enum,
                    &abs_field_enum,
                );

                key_matches.push(quote! {
                    #flat_key_enum::#field_name => #abs_path_stream
                })
            }
        } else {
            let all_key_matches: Vec<TokenStream2> = field_pair_vector
                .iter()
                .map(|field_pair| {
                    let field_name = format_ident!("{}", to_camel_case(&field_pair.top_field));
                    quote! {
                        #flat_key_enum::#field_name
                    }
                })
                .collect();

            let folder_focus_variable = folder_info.folder_focus_variable.clone();
            let get_path_function_name = format_ident!("get_{}_path", folder_focus_variable);

            key_matches.push(quote! {
                #(#all_key_matches)|* => {
                    let focus = self.#get_path_function_name();
                    panic!();
                }
            });
        }
    }

    quote! {
        #[automatically_derived]
        impl #crate_path::FocusHandler<#root_struct_description> for #focus_handler_name {
            fn get_focus_key(&self, key: #flat_key_enum) -> #abs_key_enum {
                match key {
                    #(#key_matches,)*
                }
            }

            fn get_focus_field(&self, field: #flat_field_enum) -> #abs_field_enum {
                todo!();
            }
        }
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
    let trait_impl_stream = generate_trait_impl(crate_path, data_structure);

    quote! {
        #[automatically_derived]
        impl #focus_handler_name {
            #new_stream
            #get_set_stream
        }

        #trait_impl_stream
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
