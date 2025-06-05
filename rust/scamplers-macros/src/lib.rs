use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Expr, ExprArray, ExprTuple, ItemEnum, ItemStruct, parse_macro_input};

#[proc_macro_attribute]
pub fn api_request(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let struct_item = parse_macro_input!(input as ItemStruct);
    let ItemStruct {
        ident: struct_name, ..
    } = &struct_item;

    let builder_name = format_ident!("{struct_name}Builder");
    let builder_error_name = format_ident!("{struct_name}Error");

    let output = quote! {
        #[derive(serde::Serialize, Clone, derive_builder::Builder)]
        #[builder(pattern = "owned", build_fn(error = #builder_error_name))]
        #[builder_struct_attr(wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
        #[builder_impl_attr(wasm_bindgen::prelude::wasm_bindgen)]
        #[builder_field_attr(wasm_bindgen::prelude::wasm_bindgen(readonly))]
        #[wasm_bindgen::prelude::wasm_bindgen(getter_with_clone, setter)]
        #struct_item

        #[wasm_bindgen::prelude::wasm_bindgen]
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

#[proc_macro_attribute]
pub fn api_response(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let struct_item = parse_macro_input!(input as ItemStruct);

    let output = quote! {
        #[derive(serde::Deserialize, Clone)]
        #[wasm_bindgen::prelude::wasm_bindgen(getter_with_clone, setter)]
        #struct_item
    };

    output.into()
}

fn derive_enum(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemEnum);

    let output = quote! {
        #[derive(serde::Deserialize, serde::Serialize, Default, Clone, Copy)]
        #[serde(rename_all = "snake_case")]
        #item
    };

    output.into()
}

#[proc_macro_attribute]
pub fn api_enum(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let enum_with_derives = derive_enum(input);

    let item = parse_macro_input!(enum_with_derives as ItemEnum);

    let output = quote! {
        #[wasm_bindgen::prelude::wasm_bindgen]
        #item
    };

    output.into()
}

#[proc_macro_attribute]
pub fn insert_struct(attr: TokenStream, input: TokenStream) -> TokenStream {
    let struct_item = parse_macro_input!(input as ItemStruct);

    let table_name = parse_macro_input!(attr as syn::Path);

    let output = quote! {
        #[derive(serde::Deserialize, diesel::Insertable, valuable::Valuable, garde::Validate, Debug)]
        #[diesel(table_name = #table_name, check_for_backend(diesel::pg::Pg))]
        #[garde(allow_unvalidated)]
        #struct_item
    };

    output.into()
}

#[proc_macro_attribute]
pub fn select_struct(attr: TokenStream, input: TokenStream) -> TokenStream {
    let struct_item = parse_macro_input!(input as ItemStruct);

    let table_name = parse_macro_input!(attr as syn::Path);

    let output = quote! {
        #[derive(serde::Serialize, diesel::prelude::Selectable, diesel::prelude::Queryable, valuable::Valuable, Debug)]
        #[diesel(table_name = #table_name, check_for_backend(diesel::pg::Pg))]
        #struct_item
    };

    output.into()
}

#[proc_macro_attribute]
pub fn update_struct(attr: TokenStream, input: TokenStream) -> TokenStream {
    let struct_item = parse_macro_input!(input as ItemStruct);

    let table_name = parse_macro_input!(attr as syn::Path);

    let output = quote! {
        #[derive(serde::Serialize, diesel::prelude::AsChangeset, valuable::Valuable, Debug)]
        #[diesel(table_name = #table_name, check_for_backend(diesel::pg::Pg))]
        #struct_item
    };

    output.into()
}

fn derive_query_struct(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let struct_item = parse_macro_input!(input as ItemStruct);

    let output = quote! {
        #[derive(serde::Deserialize, valuable::Valuable, Debug, garde::Validate)]
        #[garde(allow_unvalidated)]
        #struct_item
    };

    output.into()
}

#[proc_macro_attribute]
pub fn ordering_struct(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = derive_query_struct(attr, input);

    let struct_item = parse_macro_input!(input as ItemStruct);

    let output = quote! {
        #[derive(Default)]
        #struct_item
    };

    output.into()
}

#[proc_macro_attribute]
pub fn query_struct(attr: TokenStream, input: TokenStream) -> TokenStream {
    let with_derives = derive_query_struct(attr, input);

    let struct_item = parse_macro_input!(with_derives as ItemStruct);
    let name = &struct_item.ident;

    let output = quote! {
        #struct_item
        // We're relying on the fact that every query struct has a field called order_by
        impl Default for #name {
            fn default() -> Self {
                Self {
                    order_by: super::default_ordering(),
                    ..Default::default()
                }
            }
        }
    };

    output.into()
}

#[proc_macro_attribute]
pub fn db_enum(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let enum_with_derives = derive_enum(input);
    let item = parse_macro_input!(enum_with_derives as ItemEnum);

    let name = &item.ident;

    let output = quote! {
        #[derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression, Debug, strum::EnumString, strum::IntoStaticStr, valuable::Valuable)]
        #[diesel(sql_type = diesel::sql_types::Text)]
        #[strum(serialize_all = "snake_case")]
        #item

        impl diesel::deserialize::FromSql<diesel::sql_types::Text, diesel::pg::Pg> for #name {
            fn from_sql(bytes: <diesel::pg::Pg as diesel::backend::Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
                use diesel::{deserialize::FromSql, sql_types, pg::Pg};
                use std::str::FromStr;

                let string: String = FromSql::<sql_types::Text, Pg>::from_sql(bytes)?;
                Ok(Self::from_str(&string).unwrap_or_default())
            }
        }

        impl diesel::serialize::ToSql<diesel::sql_types::Text, diesel::pg::Pg> for #name {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>,
            ) -> diesel::serialize::Result {
                use diesel::{serialize::ToSql, sql_types, pg::Pg};

                let as_str: &str = self.into();
                ToSql::<sql_types::Text, Pg>::to_sql(as_str, &mut out.reborrow())
            }
        }
    };

    output.into()
}

#[proc_macro_attribute]
pub fn db_json(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemEnum);

    let name = &item.ident;

    let output = quote! {
        #[derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression, Debug, Default)]
        #[diesel(sql_type = diesel::sql_types::Text)]
        #item

        impl diesel::deserialize::FromSql<diesel::sql_types::Jsonb, diesel::pg::Pg> for #name {
            fn from_sql(bytes: <diesel::pg::Pg as diesel::backend::Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
                use diesel::{deserialize::FromSql, sql_types, pg::Pg};

                let json: serde_json::Value = FromSql::<sql_types::Jsonb, Pg>::from_sql(bytes)?;
                Ok(serde_json::from_value(json).unwrap_or_default())
            }
        }

        impl diesel::serialize::ToSql<diesel::sql_types::Jsonb, diesel::pg::Pg> for #name {
            fn to_sql<'b>(
                &'b self,
                out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>,
            ) -> diesel::serialize::Result {
                use diesel::{serialize::ToSql, sql_types, pg::Pg};

                let as_json: serde_json::to_value(self).unwrap();
                ToSql::<sql_types::Jsonb, Pg>::to_sql(&as_json, &mut out.reborrow())
            }
        }
    };

    output.into()
}

#[proc_macro_attribute]
pub fn ordinal_columns(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let enum_with_derives = derive_enum(input);

    let item = parse_macro_input!(enum_with_derives as ItemEnum);

    let output = quote! {
        #[derive(Debug, valuable::Valuable)]
        #item
    };

    output.into()
}

// This is massive and ugly and needs to be split for testability
/// Docs
/// # Panics
#[proc_macro_attribute]
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
        let snake_case_param_type = heck::AsSnekCase(param_type.to_string());

        let function_name = format_ident!("send_{snake_case_param_type}");

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
