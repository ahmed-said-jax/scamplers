#[cfg(feature = "typescript")]
use {
    crate::model::{
        institution::{Institution, NewInstitution},
        person::{NewPerson, Person},
    },
    scamplers_macros::scamplers_client,
};

#[cfg(feature = "typescript")]
#[wasm_bindgen::prelude::wasm_bindgen]
#[scamplers_client([(NewInstitution, Institution), (NewPerson, Person)])]
struct Client {
    backend_url: String,
    client: reqwest::Client,
}

#[cfg(feature = "typescript")]
#[wasm_bindgen::prelude::wasm_bindgen]
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new(backend_url: String, username: &str, password: &str) -> Self {
        use reqwest::{
            ClientBuilder,
            header::{AUTHORIZATION, HeaderMap, HeaderValue},
        };

        let mut auth = HeaderValue::from_str(&format!("{username}:{password}")).unwrap();
        auth.set_sensitive(true);

        let headers = HeaderMap::from_iter([(AUTHORIZATION, auth)]);

        let client = ClientBuilder::new()
            .default_headers(headers)
            // .http2_prior_knowledge()
            .build()
            .unwrap();

        Self {
            backend_url,
            client,
        }
    }
}
