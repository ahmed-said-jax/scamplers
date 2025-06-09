use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Expr, ExprArray, ExprTuple, ItemStruct, parse_macro_input};

// This is massive and ugly and needs to be split for testability
/// Docs
/// # Panics
pub fn scamplers_client(attr: TokenStream, input: TokenStream) -> TokenStream {
    let struct_def = parse_macro_input!(input as ItemStruct);
    let ident = &struct_def.ident;

    let ExprArray { elems, .. } = parse_macro_input!(attr as ExprArray);
    let mut method_defs = Vec::with_capacity(elems.len());
    for elem in elems {
        let Expr::Tuple(ExprTuple {
            elems: inner_elems, ..
        }) = elem
        else {
            panic!("expected array of tuples");
        };

        assert!(
            inner_elems.len() == 2,
            "expected 2 types (a data type and a return type), found {}",
            inner_elems.len()
        );

        assert!(
            inner_elems.iter().all(|e| matches!(e, Expr::Path(_))),
            "expected paths to types"
        );

        let inner_elems: Vec<_> = inner_elems
            .iter()
            .map(|e| {
                let Expr::Path(type_path) = e else {
                    panic!("expected path to type");
                };
                type_path.path.get_ident().unwrap()
            })
            .collect();

        let param_type = inner_elems[0];
        let snek_case_param_type = heck::AsSnekCase(param_type.to_string());

        let function_name = format_ident!("send_{snek_case_param_type}");

        let return_type = inner_elems[1];

        let method = quote! {
            pub async fn #function_name(&self, data: &#param_type, api_key: Option<String>) -> Result<#return_type, wasm_bindgen::JsValue> {
                self.send_request(data, api_key).await
            }
        };

        method_defs.push(method);
    }

    let output = quote! {
        #struct_def

        #[wasm_bindgen::prelude::wasm_bindgen]
        impl #ident {
            #(#method_defs)*
        }
    };

    output.into()
}
