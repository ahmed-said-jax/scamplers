use std::str::FromStr;

use argon2::password_hash::{PasswordHasher, SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::Deserialize;
use uuid::Uuid;

use crate::db;
use crate::db::person::{User, UserRow};
use crate::schema::person;

use super::ApiUser;

const API_KEY_PUBLIC_PREFIX_LEN: usize = 8;

#[derive(Deserialize)]
pub struct ApiKey(Uuid);
impl ApiKey {
    fn new() -> Self {
        Self(Uuid::new_v4())
    }

    fn hash(&self) -> String {
        let hasher = argon2::Argon2::default();
        let bytes = self.0.as_bytes();
        
        // We don't care about salting because it's already highly random, being a UUID
        let salt = SaltString::from_b64("0000").unwrap();

        let hash = hasher.hash_password(bytes, &salt).unwrap().to_string();

        hash
    }
}

impl ApiUser {
    pub async fn fetch_by_api_key(
        api_key: Uuid,
        conn: &mut AsyncPgConnection,
    ) -> super::Result<Self> {
        use person::dsl::api_key_hash;

        let hash = ApiKey(api_key).hash();

        // This is slightly dumb because first we look up the user by API key, then we look up their roles by their ID. We can do a little better
        let result = person::table
            .filter(api_key_hash.eq(hash))
            .select(UserRow::as_select())
            .first(conn)
            .await.map_err(db::Error::from);

        let Ok(user) = result else {
            match result {
                Err(db::Error::RecordNotFound) => return Err(super::Error::InvalidApiKey),
                Err(e) => return Err(super::Error::from(e)),
                Ok(_) => unreachable!()
            }
        };

        Ok(Self(user.with_roles(conn).await?))
    }
}