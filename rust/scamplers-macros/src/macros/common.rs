use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use std::collections::HashMap;
use syn::{
    Field, Fields, Ident, ItemEnum, ItemMod, ItemStruct, Type, TypePath, parse_macro_input, parse2,
};

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

trait ScamplersType {
    fn scamplers_type<'a>(
        &'a self,
        types: &'a HashMap<&Ident, &ItemStruct>,
    ) -> Option<&'a ItemStruct>;
}

impl ScamplersType for Type {
    fn scamplers_type<'a>(
        &'a self,
        types: &'a HashMap<&Ident, &ItemStruct>,
    ) -> Option<&'a ItemStruct> {
        let Type::Path(TypePath { path, .. }) = self else {
            return None;
        };

        let self_ident = path.get_ident()?;
        let inner_struct_def = types.get(self_ident)?;

        Some(inner_struct_def)
    }
}

fn create_getter(
    field_name: &Ident,
    field_type: &Type,
    access_expr: &syn::Expr,
    delegated: bool,
    wasm: bool,
) -> proc_macro2::TokenStream {
    let body = match (delegated, wasm) {
        (true, _) => quote! {#access_expr()},
        (false, true) => quote! {#access_expr.clone()},
        (false, false) => quote! {&#access_expr},
    };

    let ret_type = if wasm {
        quote! {#field_type}
    } else {
        quote! {&#field_type}
    };

    let mut method = quote! {
        pub fn #field_name(&self) -> #ret_type {
            #body
        }
    };

    if wasm {
        method = quote! {
            #[wasm_bindgen(getter)]
            #method
        }
    }

    method
}

fn impl_getters(
    parent_field_name: Option<&Ident>,
    fields: &Fields,
    structs_map: &HashMap<&Ident, &ItemStruct>,
    wasm: bool,
) -> proc_macro2::TokenStream {
    let mut getters = Vec::new();

    for Field {
        ident: field_name,
        ty: field_type,
        ..
    } in fields
    {
        let field_name = field_name.as_ref().unwrap();

        let Some(field_type_def) = field_type.scamplers_type(structs_map) else {
            let (expr, delegated) = match parent_field_name {
                Some(parent) => (quote! {self.#parent.#field_name}, true),
                None => (quote! {self.#field_name}, false),
            };

            let getter = create_getter(
                field_name,
                field_type,
                &parse2(expr).unwrap(),
                delegated,
                wasm,
            );
            getters.push(getter);

            continue;
        };

        let getter = match parent_field_name {
            Some(_) => impl_getters(parent_field_name, &field_type_def.fields, structs_map, wasm),
            None => impl_getters(Some(field_name), &field_type_def.fields, structs_map, wasm),
        };
        getters.push(getter);
    }

    quote! {
        #(#getters)*
    }
}

pub fn with_getters(input: TokenStream, wasm: bool) -> TokenStream {
    let mut mod_item = parse_macro_input!(input as ItemMod);

    let (_, mod_items) = mod_item.content.as_mut().unwrap();

    let structs: Vec<ItemStruct> = mod_items
        .iter()
        .filter_map(|i| parse2(i.to_token_stream()).ok())
        .collect();

    let structs_map = structs.iter().map(|s| (&s.ident, s)).collect();

    for struct_def in &structs {
        let struct_name = &struct_def.ident;
        let getters = impl_getters(None, &struct_def.fields, &structs_map, wasm);

        let mut impl_block = quote! {
            impl #struct_name {
                #getters
            }
        };

        if wasm {
            impl_block = quote! {
                #[wasm_bindgen::prelude::wasm_bindgen]
                #impl_block
            };
        }

        mod_items.push(parse2(impl_block).unwrap());
    }

    mod_item.to_token_stream().into()
}
