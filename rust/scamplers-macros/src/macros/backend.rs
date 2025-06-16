use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, ItemEnum, ItemImpl, ItemStruct, TypePath, parse_macro_input};

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

// pub fn summary(input: TokenStream) -> TokenStream {
//     let struct_item = parse_macro_input!(input as ItemStruct);
//     let ident = &struct_item.ident;

//     let output = quote! {
//         #struct_item

//         impl #ident {
//             pub fn id(&self) -> &uuid::Uuid {
//                 &self.reference.id
//             }

//             pub fn link(&self) -> &str {
//                 self.reference.link.as_str()
//             }
//         }
//     };

//     output.into()
// }

// pub fn detail(input: TokenStream) -> TokenStream {
//     let struct_item = parse_macro_input!(input as ItemStruct);
//     let ident = &struct_item.ident;

//     let summary_type = struct_item
//         .fields
//         .iter()
//         .find(|f| f.ident.as_ref().unwrap() == "summary")
//         .map(|f| f.ty.clone())
//         .unwrap()
//         .into_token_stream()
//         .into();

//     let summary_type = parse_macro_input!(summary_type as ItemStruct);
//     let mut getters = Vec::new();

//     for field in summary_type.fields {
//         let field_name = field.ident.as_ref().unwrap();
//         let field_type = &field.ty;

//         let getter = quote! {
//             pub fn #field_name(&self) -> &#field_type {
//                 self.summary.#field_name
//             }
//         };

//         getters.push(getter);
//     }

//     let output = quote! {
//         impl #ident {
//             #(#getters)*
//         }
//     };

//     output.into()
// }

pub fn update(attr: TokenStream, input: TokenStream) -> TokenStream {
    let struct_item = parse_macro_input!(input as ItemStruct);

    let table_name = parse_macro_input!(attr as syn::Path);

    let output = quote! {
        #[derive(serde::Deserialize, diesel::prelude::AsChangeset, diesel::prelude::Identifiable, valuable::Valuable, Debug, Default)]
        #[diesel(table_name = #table_name, check_for_backend(diesel::pg::Pg))]
        #[serde(default)]
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

fn getter_fn(path: &[Ident], return_type: TypePath) -> proc_macro2::TokenStream {
    let final_field_name = path.last().unwrap();

    let access_path = path
        .iter()
        .map(quote::ToTokens::to_token_stream)
        .reduce(|parent, child| quote! {#parent.#child});

    quote! {
        fn #final_field_name(&self) -> &return_type {
            self.#access_path
        }
    }
}

// pub fn with_getters(input: TokenStream) -> TokenStream {
//     fn impl_stub_getters(
//         outer_ident: &Ident,
//         outer_field_name: &Ident,
//     ) -> proc_macro2::TokenStream {
//         quote! {
//             impl #outer_ident {
//                 fn id(&self) -> &uuid::Uuid {
//                     &self.#outer_field_name.id()
//                 }

//                 fn link(&self) -> &str {
//                     &self.#outer_field_name.link()
//                 }
//             }
//         }
//     }

//     let mod_item = parse_macro_input!(input as ItemMod);

//     let (_, items) = mod_item.content.unwrap();

//     let struct_defs: Vec<ItemStruct> = items
//         .iter()
//         .filter_map(|i| parse2(i.to_token_stream()).ok())
//         .collect();

//     let struct_name_fields_map: HashMap<_, _> = struct_defs
//         .iter()
//         .map(|struct_def| (&struct_def.ident, struct_def))
//         .collect();

//     let mut impls = Vec::new();

//     for ItemStruct {
//         ident: outer_ident,
//         fields: outer_fields,
//         ..
//     } in &struct_defs
//     {
//         for outer_field in outer_fields {
//             let Some(outer_field_name) = &outer_field.ident else {
//                 continue;
//             };

//             let type_token_stream = outer_field.ty.to_token_stream().into();
//             let outer_field_type = parse_macro_input!(type_token_stream as syn::TypePath);
//             let Some(outer_field_type) = outer_field_type.path.get_ident() else {
//                 continue;
//             };

//             if outer_field_name == "stub" {
//                 let impl_block = quote! {
//                     impl #outer_ident {
//                         fn id(&self) -> &uuid::Uuid {
//                             &self.stub.id
//                         }

//                         fn link(&self) -> &str {
//                             &self.stub.link
//                         }
//                     }
//                 };
//                 impls.push(impl_block);

//                 break;
//             }

//             if outer_field_name == "summary" {
//                 let get_stub_fields = impl_stub_getters(outer_ident, outer_field_name);
//                 impls.push(get_stub_fields);

//                 let inner_struct_def = struct_name_fields_map.get(outer_field_type).unwrap();
//                 for Field {
//                     ident: inner_field_name,
//                     ty: inner_field_type,
//                     ..
//                 } in &inner_struct_def.fields
//                 {
//                     let Some(inner_field_name) = inner_field_name else {
//                         continue;
//                     };

//                     if inner_field_name == "stub" {
//                         continue;
//                     }

//                     let get_summary_field = quote! {
//                         impl #outer_ident {
//                             fn #inner_field_name(&self) -> &#inner_field_type {
//                                 &self.summary.#inner_field_name
//                             }
//                         }
//                     };
//                     impls.push(get_summary_field)
//                 }
//                 break;
//             }

//             if outer_field_name == "data" {
//                 let get_stub_fields = impl_stub_getters(outer_ident, outer_field_name);
//                 impls.push(get_stub_fields);

//                 let inner_struct_def = struct_name_fields_map.get(outer_field_type).unwrap();
//                 for Field {
//                     ident: inner_field_name,
//                     ty: inner_field_type,
//                     ..
//                 } in &inner_struct_def.fields
//                 {
//                     let Some(inner_field_name) = inner_field_name else {
//                         continue;
//                     };

//                     if inner_field_name == "summary" {
//                         let inner_inner_struct_def = struct_name_fields_map.get(inner_field_type)
//                     }

//                     let get_data_field = quote! {
//                         impl #outer_ident {
//                             fn #inner_field_name(&self) -> &#inner_field_type {
//                                 &self.data.#inner_field_name
//                             }
//                         }
//                     };
//                     impls.push(get_data_field)
//                 }
//                 break;
//             }
//         }
//     }

//     let output = quote! {};
//     output.into()
// }
