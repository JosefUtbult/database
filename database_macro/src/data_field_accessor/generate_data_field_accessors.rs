use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::{casing::to_camel_case, data_structure::{absolute_path::get_field_name_and_type, struct_data::{self, StructData}}, DataStructure};

fn generate_data_field_accessor(
    crate_path: &TokenStream2,
    _data_structure: &DataStructure,
    struct_data: &StructData,
) -> TokenStream2 {
    let struct_name = format_ident!("{}", struct_data.name.clone());
    let abs_key_enum = struct_data.type_names.abs_key_enum.clone();
    let abs_field_enum = struct_data.type_names.abs_field_enum.clone();

    let mut get_matches: Vec<TokenStream2> = Vec::new();
    let mut set_matches: Vec<TokenStream2> = Vec::new();

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

        let variable_name = format_ident!("{}", field.name);
        let field_name = format_ident!("{}", to_camel_case(&field.name));

        get_matches.push(quote! {
            #abs_key_enum::#field_name => #abs_field_enum::#field_name(self.#variable_name.clone())
        });

        set_matches.push(quote! {
            #abs_field_enum::#field_name(value) => self.#variable_name = value
        });
    }

    // Then, child structs
    for (_child_name, child_field_vector) in struct_data.field_to_folder_map.iter() {
        for abs_path in child_field_vector.iter() {
            let (field, _) = get_field_name_and_type(abs_path).unwrap();
            let variable_name = format_ident!("{}", field);
            let field_name = format_ident!("{}", to_camel_case(&field));

            get_matches.push(quote! {
                #abs_key_enum::#field_name(key) => #abs_field_enum::#field_name(self.#variable_name.get(key))
            });

            set_matches.push(quote! {
                #abs_field_enum::#field_name(field) => self.#variable_name.set(field)
            });
        }
    }

    quote! {
        impl #crate_path::DataFieldAccessor<#abs_key_enum, #abs_field_enum> for #struct_name {
            fn get(&self, key: #abs_key_enum) -> #abs_field_enum {
                match key {
                    #(#get_matches,)*
                }
            }

            fn set(&mut self, field: #abs_field_enum) {
                match field {
                    #(#set_matches,)*
                }
            }
        }
    }
}

fn generate_try_accessor(
    crate_path: &TokenStream2,
    _data_structure: &DataStructure,
    struct_data: &StructData,
) -> TokenStream2 {
    let struct_name = format_ident!("{}", struct_data.name.clone());
    let abs_key_enum = struct_data.type_names.abs_key_enum.clone();

    let mut result_streams: Vec<TokenStream2> = Vec::new();

    for (field_type, field_vector) in struct_data.type_to_field_map.iter() {
        let mut get_matches: Vec<TokenStream2> = Vec::new();
        let mut set_matches: Vec<TokenStream2> = Vec::new();

        let field_type = format_ident!("{}", field_type);

        for field in field_vector.iter() {
            let variable_name = format_ident!("{}", field);
            let field_name = format_ident!("{}", to_camel_case(&field));

            get_matches.push(quote! {
                #abs_key_enum::#field_name => Ok(self.#variable_name.clone())
            });

            set_matches.push(quote! {
                #abs_key_enum::#field_name => {
                    self.#variable_name = value;
                    Ok(())
                }
            });
        }

        let field_type_string = field_type.to_string();

        result_streams.push(quote! {
            impl #crate_path::DataFieldTryAccessor<#abs_key_enum, #field_type> for #struct_name {
                fn try_get(&self, key: #abs_key_enum) -> Result<#field_type, #crate_path::AccessorError> {
                    match key {
                        #(#get_matches,)*
                        _ => Err(#crate_path::AccessorError::TypeMissmatch(#field_type_string))
                    }
                }

                fn try_set(&mut self, key: #abs_key_enum, value: #field_type) -> Result<(), #crate_path::AccessorError> {
                    match key {
                        #(#set_matches,)*
                        _ => Err(#crate_path::AccessorError::TypeMissmatch(#field_type_string))
                    }
                }
            }
        });
    }

    quote! {
        #(#result_streams)*
    }
}

pub(crate) fn generate_data_field_accessors(
    crate_path: &TokenStream2,
    data_structure: &DataStructure,
    struct_data: &StructData,
) -> TokenStream2 {
    let mut res = TokenStream2::new();
    res.extend(generate_data_field_accessor(crate_path, data_structure, struct_data));
    res.extend(generate_try_accessor(crate_path, data_structure, struct_data));
    res
}
