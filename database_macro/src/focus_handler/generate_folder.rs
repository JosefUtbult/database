use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::{casing::to_camel_case, data_structure::struct_data::StructData, DataStructure};

pub(super) fn generate_folder_impl(
    crate_path: &TokenStream2,
    data_structure: &DataStructure,
    struct_data: &StructData,
) -> TokenStream2 {
    let root_struct = data_structure.root_struct.as_ref().unwrap();

    let root_namespace = root_struct.type_names.namespace.clone();
    let abs_key_enum = root_struct.type_names.abs_key_enum.clone();
    let flat_key_enum = root_struct.type_names.flat_key_enum.clone();

    let type_names = data_structure.type_names.as_ref().unwrap();
    let description_name = type_names.database_description_name.clone();
    let struct_name = format_ident!("{}", struct_data.name);

    let dynamic_key_set = quote! {
        #crate_path::DynamicKeySet<super::#root_namespace::#abs_key_enum, super::#root_namespace::#flat_key_enum>
    };

    let mut field_compare_matches: Vec<TokenStream2> = Vec::new();
    let mut folder_compare_matches: Vec<TokenStream2> = Vec::new();
    let mut field_clone_matches: Vec<TokenStream2> = Vec::new();
    let mut folder_clone_matches: Vec<TokenStream2> = Vec::new();

    for field in struct_data.fields.iter() {
        let field_name = format_ident!("{}", field.name);
        let field_variant_name = format_ident!("{}", to_camel_case(&field.name));
        if let Some(_) = data_structure.struct_map.get(&field.ty_string) {
            folder_compare_matches.push(quote! {
                self.#field_name.compare(differing_keys, &other.#field_name)?;
            });

            folder_clone_matches.push(quote! {
                self.#field_name.clone(differing_keys, &other.#field_name)?;
            });
        } else {
            field_compare_matches.push(quote! {
                if self.#field_name != other.#field_name {
                    differing_keys.insert_flat_key(super::#root_namespace::#flat_key_enum::#field_variant_name)?;
                }
            });

            field_clone_matches.push(quote! {
                if self.#field_name != other.#field_name {
                    self.#field_name = other.#field_name;
                    differing_keys.insert_flat_key(super::#root_namespace::#flat_key_enum::#field_variant_name)?;
                }
            });
        }
    }

    quote! {
        #[automatically_derived]
        impl #crate_path::Folder<super::#description_name> for super::#struct_name {
            fn compare(
                &self,
                differing_keys: &mut dyn #dynamic_key_set,
                other: &Self,
            ) -> Result<(), #crate_path::DatabaseError> {
                #(#field_compare_matches)*
                #(#folder_compare_matches)*
                Ok(())
            }

            fn clone(
                &mut self,
                differing_keys: &mut dyn #dynamic_key_set,
                other: &Self,
            ) -> Result<(), #crate_path::DatabaseError> {
                #(#field_clone_matches)*
                #(#folder_clone_matches)*
                Ok(())
            }
        }
    }
}
