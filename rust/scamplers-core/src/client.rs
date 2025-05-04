#[cfg(feature = "typescript")]
use {
    crate::model::AsEndpoint,
    crate::model::{
        institution::{Institution, NewInstitution},
        person::{CreatedUser, NewPerson, NewPerson as NewMsLogin, Person},
    },
    scamplers_macros::scamplers_client,
    serde::{Serialize, de::DeserializeOwned},
    wasm_bindgen::prelude::*,
};

#[cfg(feature = "typescript")]
#[wasm_bindgen]
#[scamplers_client([(NewInstitution, Institution), (NewPerson, Person), (NewMsLogin, CreatedUser)])]
struct Client {
    backend_url: String,
    client: reqwest::Client,
}

#[cfg(feature = "typescript")]
#[wasm_bindgen]
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new(backend_url: String, token: &str) -> Self {
        use reqwest::{
            ClientBuilder,
            header::{AUTHORIZATION, HeaderMap, HeaderValue},
        };

        let mut auth = HeaderValue::from_str(&format!("Bearer {token}")).unwrap();
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

#[cfg(feature = "typescript")]
impl Client {
    async fn send_request<Req, Resp>(
        &self,
        data: &Req,
        api_key: Option<String>,
    ) -> Result<Resp, JsValue>
    where
        Req: Serialize,
        Resp: AsEndpoint + DeserializeOwned,
    {
        use wasm_bindgen::UnwrapThrowExt;

        let Self {
            backend_url,
            client,
        } = self;

        let endpoint = Resp::as_endpoint();

        let mut request = client.post(&format!("{backend_url}{endpoint}")).json(data);

        if let Some(api_key) = api_key {
            request = request.header("X-API-Key", api_key);
        }

        let response = request
            .send()
            .await
            .unwrap_throw()
            .bytes()
            .await
            .unwrap_throw();

        let Ok(response) = serde_json::from_slice(&response) else {
            let error: serde_json::Value = serde_json::from_slice(&response).unwrap_throw();
            let error = serde_wasm_bindgen::to_value(&error).unwrap_throw();

            return Err(error);
        };

        Ok(response)
    }
}
