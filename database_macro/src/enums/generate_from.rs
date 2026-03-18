use core::panic;

use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};

use crate::{
    DataStructure,
    casing::to_camel_case,
    data_structure::{
        absolute_path::{AbsolutePath, AbsolutePathField, get_field_name_and_type},
        struct_data::{StructData, StructMap},
    },
};

pub(crate) fn generate_abs_from(
    crate_path: &TokenStream2,
    _data_structure: &DataStructure,
    struct_data: &StructData,
) -> TokenStream2 {
    let abs_key_enum = struct_data.type_names.abs_key_enum.clone();
    let abs_field_enum = struct_data.type_names.abs_field_enum.clone();

    let mut abs_field_to_abs_key_match: Vec<TokenStream2> = Vec::new();

    // Start with non-structs
    for field in struct_data.fields.iter() {
        // Ignore child struct fields
        if struct_data
            .field_to_folder_map
            .get(&field.ty_string)
            .is_some()
        {
            continue;
        }

        let field_name = format_ident!("{}", to_camel_case(&field.name));

        abs_field_to_abs_key_match.push(quote! {
            Self::#field_name(_) => #abs_key_enum::#field_name
        });
    }

    // Then, child structs
    for field in struct_data.fields.iter() {
        // Ignore non child struct fields
        if !struct_data
            .field_to_folder_map
            .get(&field.ty_string)
            .is_some()
        {
            continue;
        }

        let field_name = format_ident!("{}", to_camel_case(&field.name));

        abs_field_to_abs_key_match.push(quote! {
            Self::#field_name(field) => #abs_key_enum::#field_name(field.to_key())
        });
    }

    quote! {
        #[automatically_derived]
        impl #crate_path::ToKey<#abs_key_enum> for #abs_field_enum {
            fn to_key(&self) -> #abs_key_enum {
                match &self {
                    #(#abs_field_to_abs_key_match,)*
                }
            }
        }
    }
}

fn recurse_absolute_path(
    abs_path: &mut AbsolutePath,
    field_content: &Option<TokenStream2>,
) -> (TokenStream2, TokenStream2) {
    // Note: abs path is reversed
    if let Some(field) = abs_path.pop() {
        match field {
            AbsolutePathField::NonStruct(field_data) => {
                let field_name = format_ident!("{}", to_camel_case(&field_data.name));

                let key_stream = quote! {
                    #field_name
                };

                let field_stream = if let Some(field_content) = field_content {
                    quote! {
                        #field_name(#field_content)
                    }
                } else {
                    quote! {
                        #field_name(_)
                    }
                };

                (key_stream, field_stream)
            }
            AbsolutePathField::Struct((field_name, struct_data)) => {
                let field_name = format_ident!("{}", to_camel_case(&field_name));
                let struct_key_enum = struct_data.type_names.abs_key_enum.clone();
                let struct_field_enum = struct_data.type_names.abs_field_enum.clone();

                let (child_keys, child_fields) = recurse_absolute_path(abs_path, field_content);

                let key_stream = quote! {
                    #field_name(#struct_key_enum::#child_keys)
                };

                let field_stream = quote! {
                    #field_name(#struct_field_enum::#child_fields)
                };

                (key_stream, field_stream)
            }
        }
    } else {
        panic!("Corrupted absolute path")
    }
}

pub(crate) fn abs_path_to_token_stream(
    abs_path: &AbsolutePath,
    field_content: &Option<TokenStream2>,
    abs_key_enum: &Ident,
    abs_field_enum: &Ident,
) -> (TokenStream2, TokenStream2) {
    let mut abs_path = abs_path.clone();
    abs_path.reverse();
    let (child_keys, child_fields) = recurse_absolute_path(&mut abs_path, field_content);

    let abs_key_stream = quote! {
        #abs_key_enum::#child_keys
    };

    let abs_field_stream = quote! {
        #abs_field_enum::#child_fields
    };

    (abs_key_stream, abs_field_stream)
}

fn recurse_flat_field_from(
    struct_data: &StructData,
    struct_map: &StructMap,
    root_struct_name: &String,
    flat_key_enum: &Ident,
    flat_field_enum: &Ident,
) -> (TokenStream2, TokenStream2) {
    let mut abs_key_to_flat_key_match: Vec<TokenStream2> = Vec::new();
    let mut abs_field_to_flat_field_match: Vec<TokenStream2> = Vec::new();

    let abs_key_enum = struct_data.type_names.abs_key_enum.clone();
    let abs_field_enum = struct_data.type_names.abs_field_enum.clone();

    let (abs_key_enum_full, abs_field_enum_full) = if struct_data.name == *root_struct_name {
        (quote! {#abs_key_enum}, quote! {#abs_field_enum})
    } else {
        let namespace = struct_data.type_names.namespace.clone();
        (
            quote! {super::#namespace::#abs_key_enum},
            quote! {super::#namespace::#abs_field_enum},
        )
    };

    for field in struct_data.fields.iter() {
        let field_name = format_ident!("{}", to_camel_case(&field.name));
        if let Some(child_struct_data) = struct_map.get(&field.ty_string) {
            let (child_key_stream, child_field_stream) = recurse_flat_field_from(
                child_struct_data,
                struct_map,
                root_struct_name,
                flat_key_enum,
                flat_field_enum,
            );

            abs_key_to_flat_key_match.push(quote! {
                #abs_key_enum_full::#field_name(key) => #child_key_stream
            });

            abs_field_to_flat_field_match.push(quote! {
                #abs_field_enum_full::#field_name(field) => #child_field_stream
            });
        } else {
            abs_key_to_flat_key_match.push(quote! {
                #abs_key_enum_full::#field_name => #flat_key_enum::#field_name
            });

            abs_field_to_flat_field_match.push(quote! {
                #abs_field_enum_full::#field_name(value) => #flat_field_enum::#field_name(value)
            });
        }
    }

    let key_stream = quote! {
        match key {
            #(#abs_key_to_flat_key_match,)*
        }
    };

    let field_stream = quote! {
        match field {
            #(#abs_field_to_flat_field_match,)*
        }
    };

    (key_stream, field_stream)
}

// fn recurse_flat_folder_from(
//     struct_data: &StructData,
//     struct_map: &StructMap,
//     flat_folder_enum: &Ident,
// ) -> TokenStream2 {
//     let abs_folder_enum = struct_data.type_names.abs_folder_enum.clone();
//     let mut abs_folder_to_flat_folder_match: Vec<TokenStream2> = Vec::new();
//     for (folder_type, field_vector) in struct_data.field_to_folder_map.iter() {
//         if let Some(child_struct_data) = struct_map.get(folder_type) {
//             for abs_path in field_vector.first().iter() {
//                 let (field_name, _) = get_field_name_and_type(abs_path).unwrap();

//                 let field_name_ident = format_ident!("{}", to_camel_case(&field_name));
//                 let folder_type = format_ident!("{}", to_camel_case(&folder_type));

//                 abs_folder_to_flat_folder_match.push(quote! {
//                     #abs_folder_enum::#field_name_ident => #flat_folder_enum::#folder_type
//                 });

//                 if !child_struct_data.field_to_folder_map.is_empty() {
//                     let inner_field_name = format_ident!("In{}", to_camel_case(&field_name));

//                     let child_folder_stream =
//                         recurse_flat_folder_from(child_struct_data, struct_map, flat_folder_enum);

//                     abs_folder_to_flat_folder_match.push(quote! {
//                         #abs_folder_enum::#inner_field_name(folder) => #child_folder_stream
//                     });
//                 }
//             }
//         }
//     }

//     quote! {
//         match folder {
//             #(#abs_folder_to_flat_folder_match,)*
//         }
//     }
// }

pub(crate) fn generate_flat_from(
    crate_path: &TokenStream2,
    data_structure: &DataStructure,
) -> TokenStream2 {
    let root_struct = data_structure.root_struct.as_ref().unwrap();

    let abs_key_enum = root_struct.type_names.abs_key_enum.clone();
    let abs_field_enum = root_struct.type_names.abs_field_enum.clone();

    let flat_key_enum = root_struct.type_names.flat_key_enum.clone();
    let flat_field_enum = root_struct.type_names.flat_field_enum.clone();

    let mut flat_field_to_flat_key_match: Vec<TokenStream2> = Vec::new();

    for (flat_field, _) in root_struct.field_to_field_abs_path_map.iter() {
        let field_name = format_ident!("{}", to_camel_case(&flat_field));

        flat_field_to_flat_key_match.push(quote! {
            Self::#field_name(_) => #flat_key_enum::#field_name
        });
    }

    let (abs_key_to_flat_key_match, abs_field_to_flat_field_match) = recurse_flat_field_from(
        root_struct,
        &data_structure.struct_map,
        &root_struct.name,
        &flat_key_enum,
        &flat_field_enum,
    );

    quote! {
        #[automatically_derived]
        impl #crate_path::ToKey<#flat_key_enum> for #flat_field_enum {
            fn to_key(&self) -> #flat_key_enum {
                match self {
                    #(#flat_field_to_flat_key_match,)*
                }
            }
        }

        #[automatically_derived]
        impl From<#abs_key_enum> for #flat_key_enum {
            fn from(key: #abs_key_enum) -> Self {
                #abs_key_to_flat_key_match
            }
        }

        #[automatically_derived]
        impl From<#abs_field_enum> for #flat_field_enum {
            fn from(field: #abs_field_enum) -> Self {
                #abs_field_to_flat_field_match
            }
        }
    }
}
