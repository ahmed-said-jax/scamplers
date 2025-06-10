use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemEnum, ItemImpl, ItemStruct, parse_macro_input};

use super::common::{derive_enum, impl_query_request_default};

fn wasm_builder(input: TokenStream, with_default: bool) -> TokenStream {
    let struct_item = parse_macro_input!(input as ItemStruct);
    let ItemStruct {
        ident: struct_name, ..
    } = &struct_item;

    let builder_name = format_ident!("{struct_name}Builder");
    let builder_error_name = format_ident!("{struct_name}Error");

    let first_lines = if with_default {
        quote! {
            #[derive(serde::Serialize, Clone, derive_builder::Builder, Default)]
            #[builder(pattern = "owned", build_fn(error = #builder_error_name), default)]
        }
    } else {
        quote! {
            #[derive(serde::Serialize, Clone, derive_builder::Builder)]
            #[builder(pattern = "owned", build_fn(error = #builder_error_name))]
        }
    };

    let output = quote! {
        #first_lines
        #[builder_struct_attr(wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
        #[builder_impl_attr(wasm_bindgen::prelude::wasm_bindgen)]
        #[builder_field_attr(wasm_bindgen::prelude::wasm_bindgen(readonly))]
        #[wasm_bindgen::prelude::wasm_bindgen(getter_with_clone, setter, inspectable)]
        #struct_item

        #[wasm_bindgen::prelude::wasm_bindgen(getter_with_clone)]
        struct #builder_error_name(String);

        impl From<derive_builder::UninitializedFieldError> for #builder_error_name {
            fn from(err: derive_builder::UninitializedFieldError) -> #builder_error_name {
                #builder_error_name(err.field_name().to_string())
            }
        }

        #[wasm_bindgen::prelude::wasm_bindgen]
        impl #builder_error_name {
            pub fn error(&self) -> String {
                let Self(field) = self;
                format!("{field} must be set")
            }
        }

        #[wasm_bindgen::prelude::wasm_bindgen]
        impl #struct_name {
            pub fn new() -> #builder_name {
                #builder_name::default()
            }
        }
    };

    output.into()
}

pub fn write_request(input: TokenStream) -> TokenStream {
    wasm_builder(input, false)
}

pub fn ordering(input: TokenStream) -> TokenStream {
    wasm_builder(input, true)
}

pub fn query_request(input: TokenStream) -> TokenStream {
    let cloned = input.clone();
    let struct_item = parse_macro_input!(cloned as ItemStruct);

    let ItemStruct {
        ident: struct_name, ..
    } = &struct_item;

    let default_impl = impl_query_request_default(input);
    let default_impl = parse_macro_input!(default_impl as ItemImpl);

    let output = quote! {
        #[derive(serde::Serialize)]
        #[wasm_bindgen::prelude::wasm_bindgen(getter_with_clone, setter, inspectable)]
        #struct_item

        #default_impl

        #[wasm_bindgen::prelude::wasm_bindgen]
        impl #struct_name {
            #[wasm_bindgen::prelude::wasm_bindgen(constructor)]
            pub fn new() -> Self {
                Self::default()
            }
        }
    };

    output.into()
}

pub fn response(input: TokenStream) -> TokenStream {
    let struct_item = parse_macro_input!(input as ItemStruct);

    let output = quote! {
        #[derive(serde::Deserialize, Clone)]
        #[wasm_bindgen::prelude::wasm_bindgen(getter_with_clone, setter)]
        #struct_item
    };

    output.into()
}

pub fn enum_(input: TokenStream) -> TokenStream {
    let enum_with_derives = derive_enum(input);

    let enum_item = parse_macro_input!(enum_with_derives as ItemEnum);

    let output = quote! {
        #[wasm_bindgen::prelude::wasm_bindgen]
        #enum_item
    };

    output.into()
}
