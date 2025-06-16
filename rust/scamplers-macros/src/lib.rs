use proc_macro::TokenStream;

mod macros;
use macros::{backend, client, frontend};

#[proc_macro_attribute]
pub fn frontend_insertion(_attr: TokenStream, input: TokenStream) -> TokenStream {
    frontend::insertion(input)
}

#[proc_macro_attribute]
pub fn frontend_ordering(_attr: TokenStream, input: TokenStream) -> TokenStream {
    frontend::ordering(input)
}

#[proc_macro_attribute]
pub fn frontend_query_request(_attr: TokenStream, input: TokenStream) -> TokenStream {
    frontend::query_request(input)
}

#[proc_macro_attribute]
pub fn frontend_response(_attr: TokenStream, input: TokenStream) -> TokenStream {
    frontend::response(input)
}

#[proc_macro_attribute]
pub fn frontend_update(_attr: TokenStream, input: TokenStream) -> TokenStream {
    frontend::update(input)
}

#[proc_macro_attribute]
pub fn frontend_enum(_attr: TokenStream, input: TokenStream) -> TokenStream {
    frontend::enum_(input)
}

#[proc_macro_attribute]
pub fn backend_insertion(attr: TokenStream, input: TokenStream) -> TokenStream {
    backend::insertion(attr, input)
}

#[proc_macro_attribute]
pub fn backend_selection(attr: TokenStream, input: TokenStream) -> TokenStream {
    backend::selection(attr, input)
}

// #[proc_macro_attribute]
// pub fn backend_summary(_attr: TokenStream, input: TokenStream) -> TokenStream {
//     backend::summary(input)
// }

// #[proc_macro_attribute]
// pub fn backend_detail(_attr: TokenStream, input: TokenStream) -> TokenStream {
//     backend::detail(input)
// }

#[proc_macro_attribute]
pub fn backend_ordering(_attr: TokenStream, input: TokenStream) -> TokenStream {
    backend::ordering(input)
}

#[proc_macro_attribute]
pub fn backend_query_request(_attr: TokenStream, input: TokenStream) -> TokenStream {
    backend::query_request(input)
}

#[proc_macro_attribute]
pub fn backend_update(attr: TokenStream, input: TokenStream) -> TokenStream {
    backend::update(attr, input)
}

#[proc_macro_attribute]
pub fn backend_ordinal_columns_enum(_attr: TokenStream, input: TokenStream) -> TokenStream {
    backend::ordinal_columns_enum(input)
}

#[proc_macro_attribute]
pub fn backend_db_enum(_attr: TokenStream, input: TokenStream) -> TokenStream {
    backend::db_enum(input)
}

#[proc_macro_attribute]
pub fn backend_db_json(_attr: TokenStream, input: TokenStream) -> TokenStream {
    backend::db_json(input)
}

// This is massive and ugly and needs to be split for testability
/// Docs
/// # Panics
#[proc_macro_attribute]
pub fn scamplers_client(attr: TokenStream, input: TokenStream) -> TokenStream {
    client::scamplers_client(attr, input)
}
