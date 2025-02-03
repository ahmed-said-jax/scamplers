use std::str::FromStr;

use argon2::password_hash::{PasswordHasher, SaltString, rand_core::OsRng};
use serde::Deserialize;
use uuid::Uuid;

const API_KEY_PUBLIC_PREFIX_LEN: usize = 8;

pub trait AsApiKey {
    fn prefix(&self) -> String; // should this return something more general so we don't have to clone
}

#[derive(Deserialize)]
pub struct ApiKeyHash2 {
    prefix: String,
    pub hash: String,
}
impl ApiKeyHash2 {
    pub fn new() -> Self {
        let hasher = argon2::Argon2::default();

        let salt = SaltString::generate(&mut OsRng);
        let uuid = Uuid::new_v4();
        let uuid = uuid.as_bytes();

        // unwraps is fine because we don't expect this to fail
        let hash = hasher.hash_password(uuid, &salt).unwrap().to_string();

        let prefix =
            String::from_utf8(uuid[0..API_KEY_PUBLIC_PREFIX_LEN].to_ascii_lowercase()).unwrap();

        Self { prefix, hash }
    }
}

impl AsApiKey for ApiKeyHash2 {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
}

impl FromStr for ApiKeyHash2 {
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

pub struct ApiKeyHash(String, String);
impl ApiKeyHash {
    pub fn new() -> Self {
        let hasher = argon2::Argon2::default();

        let salt = SaltString::generate(&mut OsRng);
        let uuid = Uuid::new_v4();
        let uuid = uuid.as_bytes();

        // unwraps is fine because we don't expect this to fail
        let hash = hasher.hash_password(uuid, &salt).unwrap().to_string();

        let prefix =
            String::from_utf8(uuid[0..API_KEY_PUBLIC_PREFIX_LEN].to_ascii_lowercase()).unwrap();

        Self(prefix, hash)
    }

    pub fn hashed_part(&self) -> &str {
        &self.1
    }
}

impl AsApiKey for ApiKeyHash {
    fn prefix(&self) -> String {
        self.0.clone()
    }
}

impl FromStr for ApiKeyHash {
    type Err = super::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(".").collect();

        let (Some(prefix), Some(hash)) = (parts.get(0), parts.get(1)) else {
            return Err(super::Error::InvalidApiKey);
        };

        Ok(Self(prefix.to_string(), hash.to_string()))
    }
}

impl AsApiKey for Uuid {
    fn prefix(&self) -> String {
        let prefix = self.as_bytes()[0..API_KEY_PUBLIC_PREFIX_LEN].to_ascii_lowercase();

        String::from_utf8(prefix).unwrap()
    }
}
