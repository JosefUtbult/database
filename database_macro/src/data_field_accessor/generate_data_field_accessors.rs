use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::{
    DataStructure, casing::to_camel_case, data_structure::struct_data::StructData,
    enums::abs_path_to_token_stream,
};

pub(crate) fn generate_data_field_accessors(
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
            .field_to_child_struct_map
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
    for (_child_name, child_field_vector) in struct_data.field_to_child_struct_map.iter() {
        for field in child_field_vector.iter() {
            let variable_name = format_ident!("{}", field);
            let field_name = format_ident!("{}", to_camel_case(field));

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
