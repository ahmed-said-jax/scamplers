use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemEnum, ItemImpl, ItemStruct, parse_macro_input};

use super::common::{derive_enum, impl_query_request_default};

pub fn insertion(attr: TokenStream, input: TokenStream) -> TokenStream {
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

pub fn selection(attr: TokenStream, input: TokenStream) -> TokenStream {
    let struct_item = parse_macro_input!(input as ItemStruct);

    let table_name = parse_macro_input!(attr as syn::Path);

    let output = quote! {
        #[derive(serde::Serialize, diesel::prelude::Selectable, diesel::prelude::Queryable, valuable::Valuable, Debug)]
        #[diesel(table_name = #table_name, check_for_backend(diesel::pg::Pg))]
        #struct_item
    };

    output.into()
}

pub fn update(attr: TokenStream, input: TokenStream) -> TokenStream {
    let struct_item = parse_macro_input!(input as ItemStruct);

    let table_name = parse_macro_input!(attr as syn::Path);

    let output = quote! {
        #[derive(serde::Serialize, diesel::prelude::AsChangeset, valuable::Valuable, Debug)]
        #[diesel(table_name = #table_name, check_for_backend(diesel::pg::Pg))]
        #struct_item
    };

    output.into()
}

pub fn ordering(input: TokenStream) -> TokenStream {
    let struct_item = parse_macro_input!(input as ItemStruct);

    let output = quote! {
        #[derive(serde::Deserialize, valuable::Valuable, garde::Validate, Debug, Default)]
        #[serde(default)]
        #[garde(allow_unvalidated)]
        #struct_item
    };

    output.into()
}

pub fn query_request(input: TokenStream) -> TokenStream {
    let cloned = input.clone();
    let struct_item = parse_macro_input!(cloned as ItemStruct);

    let default_impl = impl_query_request_default(input);
    let default_impl = parse_macro_input!(default_impl as ItemImpl);

    let output = quote! {
        #[derive(serde::Deserialize, valuable::Valuable, garde::Validate, Debug)]
        #[serde(default)]
        #[garde(allow_unvalidated)]
        #struct_item

        #default_impl
    };

    output.into()
}

pub fn ordinal_columns_enum(input: TokenStream) -> TokenStream {
    let enum_with_derives = derive_enum(input);

    let enum_item = parse_macro_input!(enum_with_derives as ItemEnum);

    let output = quote! {
        #[derive(Debug, valuable::Valuable)]
        #enum_item
    };

    output.into()
}

pub fn db_enum(input: TokenStream) -> TokenStream {
    let enum_with_derives = derive_enum(input);
    let enum_item = parse_macro_input!(enum_with_derives as ItemEnum);

    let ItemEnum { ident, .. } = &enum_item;

    let output = quote! {
        #[derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression, Debug, strum::EnumString, strum::IntoStaticStr, valuable::Valuable)]
        #[diesel(sql_type = diesel::sql_types::Text)]
        #[strum(serialize_all = "snake_case")]
        #enum_item

        impl diesel::deserialize::FromSql<diesel::sql_types::Text, diesel::pg::Pg> for #ident {
            fn from_sql(bytes: <diesel::pg::Pg as diesel::backend::Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
                use diesel::{deserialize::FromSql, sql_types, pg::Pg};
                use std::str::FromStr;

                let string: String = FromSql::<sql_types::Text, Pg>::from_sql(bytes)?;
                Ok(Self::from_str(&string).unwrap_or_default())
            }
        }

        impl diesel::serialize::ToSql<diesel::sql_types::Text, diesel::pg::Pg> for #ident {
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

pub fn db_json(input: TokenStream) -> TokenStream {
    let struct_item = parse_macro_input!(input as ItemStruct);

    let ItemStruct { ident, .. } = &struct_item;

    let output = quote! {
        #[derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression, Debug, Default)]
        #[diesel(sql_type = diesel::sql_types::Text)]
        #struct_item

        impl diesel::deserialize::FromSql<diesel::sql_types::Jsonb, diesel::pg::Pg> for #ident {
            fn from_sql(bytes: <diesel::pg::Pg as diesel::backend::Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
                use diesel::{deserialize::FromSql, sql_types, pg::Pg};

                let json: serde_json::Value = FromSql::<sql_types::Jsonb, Pg>::from_sql(bytes)?;
                Ok(serde_json::from_value(json).unwrap_or_default())
            }
        }

        impl diesel::serialize::ToSql<diesel::sql_types::Jsonb, diesel::pg::Pg> for #ident {
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
