use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, Field, Fields, FieldsNamed, ItemEnum, ItemStruct, parse_macro_input};

#[proc_macro_attribute]
pub fn api_request(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let struct_item = parse_macro_input!(input as ItemStruct);

    let ItemStruct { ident, fields, .. } = &struct_item;

    let mut constructor_params = quote! {};
    constructor_params.extend(fields.iter().map(|f| {
        let Field {
            ident: Some(ident),
            ty,
            ..
        } = f
        else {
            panic!("struct fields must be named");
        };
        quote! {#ident: #ty,}
    }));

    let mut constructor_body = quote! {};
    constructor_body.extend(fields.iter().map(|f| {
        let field_ident = f.ident.as_ref().unwrap();
        quote! {#field_ident,}
    }));

    let output = quote! {
        #[derive(serde::Serialize, Clone)]
        #[wasm_bindgen::prelude::wasm_bindgen(getter_with_clone, setter)]
        #struct_item

        #[wasm_bindgen::prelude::wasm_bindgen]
        impl #ident {
            pub fn to_json(&self) -> String {
                use wasm_bindgen::prelude::*;
                serde_json::to_string(self).unwrap_throw()
            }

            #[wasm_bindgen::prelude::wasm_bindgen(constructor)]
            pub fn new(#constructor_params) -> Self {
                Self {
                    #constructor_body
                }
            }
        }
    };

    output.into()
}

#[proc_macro_attribute]
pub fn api_response(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let struct_item = parse_macro_input!(input as ItemStruct);

    let ItemStruct { ident, .. } = &struct_item;

    let output = quote! {
        #[derive(serde::Deserialize, Clone)]
        #[wasm_bindgen::prelude::wasm_bindgen(getter_with_clone, setter)]
        #struct_item

        #[wasm_bindgen::prelude::wasm_bindgen]
        impl #ident {
            pub fn from_json(json: &str) -> Self {
                use wasm_bindgen::prelude::*;
                serde_json::from_str(json).unwrap_throw()
            }
        }
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
pub fn filter_struct(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let struct_item = parse_macro_input!(input as ItemStruct);

    let output = quote! {
        #[derive(serde::Deserialize, valuable::Valuable, Debug, Default)]
        #struct_item
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
