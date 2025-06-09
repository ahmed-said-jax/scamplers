use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemEnum, ItemStruct, parse_macro_input};

pub(super) fn derive_enum(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemEnum);

    let output = quote! {
        #[derive(serde::Deserialize, serde::Serialize, Default, Clone, Copy)]
        #[serde(rename_all = "snake_case")]
        #item
    };

    output.into()
}

pub(super) fn impl_query_request_default(input: TokenStream) -> TokenStream {
    let struct_item = parse_macro_input!(input as ItemStruct);
    let ItemStruct { ident, fields, .. } = &struct_item;

    let non_ordering_fields = fields
        .iter()
        .filter_map(|f| f.ident.as_ref())
        .filter(|i| *i != "order_by")
        .map(|i| quote! {#i: Default::default()});

    let output = quote! {
        // We're relying on the fact that every query struct has a field called order_by and that there's a trait called DefaultOrdering in the crate
        impl Default for #ident {
            fn default() -> Self {
                Self {
                    #(#non_ordering_fields),*,
                    order_by: crate::model::DefaultOrdering::default(),
                }
            }
        }
    };

    output.into()
}
