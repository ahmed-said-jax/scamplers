use std::str::FromStr;

use argon2::password_hash::{PasswordHasher, SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::Deserialize;
use uuid::Uuid;

use crate::db;
use crate::db::person::User;
use crate::schema::person;

use super::ApiUser;

const API_KEY_PUBLIC_PREFIX_LEN: usize = 8;

pub trait AsApiKey {
    fn prefix(&self) -> String; // should this return something more general so we don't have to clone
}

#[derive(Deserialize)]
pub struct ApiKey {
    prefix: String,
    pub hash: String,
}
impl ApiKey {
    pub fn new() -> Self {
        let hasher = argon2::Argon2::default();

        let salt = SaltString::generate(&mut OsRng);
        let uuid = Uuid::new_v4();
        let uuid = uuid.as_bytes();

        // unwrap is fine because we don't expect this to fail
        let hash = hasher.hash_password(uuid, &salt).unwrap().to_string();

        let prefix =
            String::from_utf8(uuid[0..API_KEY_PUBLIC_PREFIX_LEN].to_ascii_lowercase()).unwrap();

        Self { prefix, hash }
    }
}

impl AsApiKey for ApiKey {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
}

impl FromStr for ApiKey {
    type Err = super::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(".").collect();

        let (Some(prefix), Some(hash)) = (parts.get(0), parts.get(1)) else {
            return Err(super::Error::InvalidApiKey);
        };

        Ok(Self {
            prefix: prefix.to_string(),
            hash: hash.to_string(),
        })
    }
}

impl AsApiKey for Uuid {
    fn prefix(&self) -> String {
        let prefix = self.as_bytes()[0..API_KEY_PUBLIC_PREFIX_LEN].to_ascii_lowercase();

        String::from_utf8(prefix).unwrap()
    }
}

impl ApiUser {
    pub async fn fetch_by_api_key(
        api_key: Uuid,
        conn: &mut AsyncPgConnection,
    ) -> super::Result<Self> {
        use person::dsl::api_key_hash;

        let prefix = api_key.prefix();

        let result = person::table
            .filter(api_key_hash.retrieve_as_text("prefix").eq(prefix))
            .select((User::as_select(), api_key_hash.assume_not_null()))
            .first(conn)
            .await.map_err(db::Error::from);

        let Ok((user, found_api_key)) = result else {
            match result {
                Err(db::Error::RecordNotFound) => return Err(super::Error::InvalidApiKey),
                Err(e) => return Err(super::Error::from(e)),
                Ok(_) => unreachable!()
            }
        };

        let found_api_key: ApiKey = serde_json::from_value(found_api_key).unwrap();
        let found_api_key_hash = PasswordHash::new(&found_api_key.hash).unwrap();

        Argon2::default()
            .verify_password(api_key.as_bytes(), &found_api_key_hash)
            .map_err(|_| super::Error::InvalidApiKey)?;

        Ok(Self(user))
    }
}