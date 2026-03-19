use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};

use crate::{
    DataStructure,
    casing::to_camel_case,
    data_structure::{
        self,
        absolute_path::{AbsolutePath, AbsolutePathField},
        struct_data::{self, StructData},
    },
    focus_handler::generate_folder::generate_folder_impl,
};

fn generate_folder_path_enum(
    _crate_path: &TokenStream2,
    data_structure: &DataStructure,
    _struct_data: &StructData,
    folder_name: &String,
    path_vector: &Vec<AbsolutePath>,
) -> TokenStream2 {
    let mut enum_variant_matches: Vec<TokenStream2> = Vec::new();
    for path in path_vector.iter() {
        let first_field = path.first().unwrap();
        match first_field {
            AbsolutePathField::NonStruct(field_data) => {
                let field_name = format_ident!("{}", to_camel_case(&field_data.name));
                enum_variant_matches.push(quote! {
                    #field_name
                });
            }
            AbsolutePathField::Struct((field_name, sub_folder_data)) => {
                let field_name = format_ident!("{}", to_camel_case(field_name));
                let sub_folder_name = &sub_folder_data.name;

                if sub_folder_name == folder_name {
                    enum_variant_matches.push(quote! {
                        #field_name
                    });
                } else {
                    let sub_folder_data = data_structure.struct_map.get(sub_folder_name).unwrap();
                    let sub_folder_namespace = sub_folder_data.type_names.namespace.clone();

                    let folder_namespace = data_structure
                        .struct_map
                        .get(folder_name)
                        .unwrap()
                        .type_names
                        .namespace
                        .clone();

                    enum_variant_matches.push(quote! {
                        #field_name(super::super::#sub_folder_namespace::#folder_namespace::Folder)
                    });
                }
            }
        }
    }

    quote! {
        pub enum Folder {
            #(#enum_variant_matches,)*
        }
    }
}

fn generate_folder_handler_impl(
    crate_path: &TokenStream2,
    data_structure: &DataStructure,
    struct_data: &StructData,
    folder_name: &String,
    path_vector: &Vec<AbsolutePath>,
) -> TokenStream2 {
    let struct_name = format_ident!("{}", struct_data.name);
    let struct_name = quote! {
        super::super::#struct_name
    };

    let folder_name_ident = format_ident!("{}", folder_name);
    let folder_name_ident = quote! {
        super::super::#folder_name_ident
    };

    let root_struct = data_structure.root_struct.as_ref().unwrap();
    let root_namespace = root_struct.type_names.namespace.clone();
    let abs_key_enum = root_struct.type_names.abs_key_enum.clone();
    let flat_key_enum = root_struct.type_names.flat_key_enum.clone();

    let type_names = data_structure.type_names.as_ref().unwrap();
    let description_name = type_names.database_description_name.clone();

    let description_name = quote! {
        super::super::#description_name
    };

    let dynamic_key_set = quote! {
        #crate_path::DynamicKeySet<super::super::#root_namespace::#abs_key_enum, super::super::#root_namespace::#flat_key_enum>
    };

    let mut get_at_matches: Vec<TokenStream2> = Vec::new();
    let mut get_at_mut_matches: Vec<TokenStream2> = Vec::new();
    let mut compare_matches: Vec<TokenStream2> = Vec::new();
    let mut clone_matches: Vec<TokenStream2> = Vec::new();
    for path in path_vector.iter() {
        let first_field = path.first().unwrap();
        match first_field {
            AbsolutePathField::NonStruct(_) => {}
            AbsolutePathField::Struct((field_name, sub_folder_data)) => {
                let field_name_ident = format_ident!("{}", field_name);
                let variant_name = format_ident!("{}", to_camel_case(field_name));
                let sub_folder_name = &sub_folder_data.name;

                if sub_folder_name == folder_name {
                    get_at_matches.push(quote! {
                        Folder::#variant_name => &self.#field_name_ident
                    });

                    get_at_mut_matches.push(quote! {
                        Folder::#variant_name => &mut self.#field_name_ident
                    });

                    compare_matches.push(quote! {
                        Folder::#variant_name => self.#field_name_ident.compare(differing_keys, other)
                    });

                    clone_matches.push(quote! {
                        Folder::#variant_name => self.#field_name_ident.clone(differing_keys, other)
                    });
                } else {
                    get_at_matches.push(quote! {
                        Folder::#variant_name(path) => self.#field_name_ident.get_at(path)
                    });

                    get_at_mut_matches.push(quote! {
                        Folder::#variant_name(path) => self.#field_name_ident.get_at_mut(path)
                    });

                    compare_matches.push(quote! {
                        Folder::#variant_name(path) => self.#field_name_ident.compare_path(differing_keys, path, other)
                    });

                    clone_matches.push(quote! {
                        Folder::#variant_name(path) => self.#field_name_ident.clone_path(differing_keys, path, other)
                    });
                }
            }
        }
    }

    quote! {
        #[automatically_derived]
        impl #crate_path::FolderHandler<Folder, #description_name> for #struct_name {
            type Content = #folder_name_ident;

            fn get_at<'a>(
                &'a self,
                path: Folder,
            ) -> &'a Self::Content {
                match path {
                    #(#get_at_matches,)*
                }
            }

            fn get_at_mut<'a>(
                &'a mut self,
                path: Folder,
            ) -> &'a mut Self::Content {
                match path {
                    #(#get_at_mut_matches,)*
                }
            }

            fn compare_path(
                &self,
                differing_keys: &mut dyn #dynamic_key_set,
                path: Folder,
                other: &Self::Content,
            ) -> Result<(), #crate_path::DatabaseError> {
                match path {
                    #(#compare_matches,)*
                }
            }

            fn clone_path(
                &mut self,
                differing_keys: &mut dyn #dynamic_key_set,
                path: Folder,
                other: &Self::Content,
            ) -> Result<(), #crate_path::DatabaseError> {
                match path {
                    #(#clone_matches,)*
                }
            }
        }
    }
}

fn generate_folder_to_full(
    crate_path: &TokenStream2,
    data_structure: &DataStructure,
    struct_data: &StructData,
    folder_name: &String,
    path_vector: &Vec<AbsolutePath>,
) -> TokenStream2 {
    let namespace = struct_data.type_names.namespace.clone();

    let abs_key_enum = struct_data.type_names.abs_key_enum.clone();
    let abs_key_enum = quote! {
        super::super::#namespace::#abs_key_enum
    };

    let abs_field_enum = struct_data.type_names.abs_field_enum.clone();
    let abs_field_enum = quote! {
        super::super::#namespace::#abs_field_enum
    };

    let folder_namespace = data_structure
        .struct_map
        .get(folder_name)
        .unwrap()
        .type_names
        .namespace
        .clone();

    let folder_abs_key_enum = data_structure
        .struct_map
        .get(folder_name)
        .unwrap()
        .type_names
        .abs_key_enum
        .clone();

    let folder_abs_key_enum = quote! {
        super::super::#folder_namespace::#folder_abs_key_enum
    };

    let folder_abs_field_enum = data_structure
        .struct_map
        .get(folder_name)
        .unwrap()
        .type_names
        .abs_field_enum
        .clone();

    let folder_abs_field_enum = quote! {
        super::super::#folder_namespace::#folder_abs_field_enum
    };

    let mut build_full_key_matches: Vec<TokenStream2> = Vec::new();
    let mut build_full_field_matches: Vec<TokenStream2> = Vec::new();
    for path in path_vector.iter() {
        let first_field = path.first().unwrap();
        match first_field {
            AbsolutePathField::NonStruct(_) => {}
            AbsolutePathField::Struct((field_name, sub_folder_data)) => {
                let variant_name = format_ident!("{}", to_camel_case(field_name));
                let sub_folder_name = &sub_folder_data.name;

                if sub_folder_name == folder_name {
                    build_full_key_matches.push(quote! {
                        Self::#variant_name => #abs_key_enum::#variant_name(internal)
                    });

                    build_full_field_matches.push(quote! {
                        Self::#variant_name => #abs_field_enum::#variant_name(internal)
                    });
                } else {
                    build_full_key_matches.push(quote! {
                        Self::#variant_name(path) => #abs_key_enum::#variant_name(path.build_full(internal))
                    });

                    build_full_field_matches.push(quote! {
                        Self::#variant_name(path) => #abs_field_enum::#variant_name(path.build_full(internal))
                    });
                }
            }
        }
    }

    quote! {
        #[automatically_derived]
        impl #crate_path::ToFull<#abs_key_enum, #folder_abs_key_enum> for Folder {
            fn build_full(&self, internal: #folder_abs_key_enum) -> #abs_key_enum {
                match self {
                    #(#build_full_key_matches,)*
                }
            }
        }

        #[automatically_derived]
        impl #crate_path::ToFull<#abs_field_enum, #folder_abs_field_enum> for Folder {
            fn build_full(&self, internal: #folder_abs_field_enum) -> #abs_field_enum {
                match self {
                    #(#build_full_field_matches,)*
                }
            }
        }
    }
}

pub(crate) fn generate_folder_handler(
    crate_path: &TokenStream2,
    data_structure: &DataStructure,
    struct_data: &StructData,
) -> TokenStream2 {
    let mut result = TokenStream2::new();

    result.extend(generate_folder_impl(
        crate_path,
        data_structure,
        struct_data,
    ));

    for (folder_name, path_vector) in struct_data.field_to_folder_map.iter() {
        let folder_namespace = data_structure
            .struct_map
            .get(folder_name)
            .unwrap()
            .type_names
            .namespace
            .clone();

        let folder_path_enum_stream = generate_folder_path_enum(
            crate_path,
            data_structure,
            struct_data,
            folder_name,
            path_vector,
        );

        let folder_to_full_stream = generate_folder_to_full(
            crate_path,
            data_structure,
            struct_data,
            folder_name,
            path_vector,
        );

        let folder_handler_impl_stream = generate_folder_handler_impl(
            crate_path,
            data_structure,
            struct_data,
            folder_name,
            path_vector,
        );

        result.extend(quote! {
            pub mod #folder_namespace {
                #[allow(unused_imports)]
                use #crate_path::Folder as _;
                #folder_path_enum_stream
                #folder_to_full_stream
                #folder_handler_impl_stream
            }
        });
    }

    result
}
