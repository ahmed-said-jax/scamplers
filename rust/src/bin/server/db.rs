use std::{fmt::Display, str::FromStr};

use argon2::password_hash::{SaltString, PasswordHasher};
use diesel::result::DatabaseErrorInformation;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool};
use regex::Regex;
use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;
use valuable::Valuable;

mod person;
mod institution;
mod index_sets;

pub trait Create: Send {
    type Returns: Send;

    fn create(
        &self,
        conn: &mut AsyncPgConnection,
    ) -> impl Future<Output = Result<Self::Returns>> + Send;
}

pub trait Read: Serialize + Sized + Send {
    type Id: Send + Display;
    type Filter: Sync + Send + Paginate;

    fn fetch_many(
        filter: Self::Filter,
        conn: &mut AsyncPgConnection,
    ) -> impl Future<Output = Result<Vec<Self>>> + Send;

    fn fetch_by_id(
        id: Self::Id,
        conn: &mut AsyncPgConnection,
    ) -> impl Future<Output = Result<Self>> + Send;
}

pub trait Update {
    type Returns;

    async fn update(&self, conn: &mut AsyncPgConnection) -> Result<Self::Returns>;
}

impl<T: Update> Update for Vec<T> {
    type Returns = Vec<T::Returns>;

    async fn update(&self, conn: &mut AsyncPgConnection) -> Result<Self::Returns> {
        let mut results = Vec::with_capacity(self.len());

        for item in self {
            results.push(item.update(conn).await?)
        }

        Ok(results)
    }
}

pub trait ReadRelatives<T: Read>: DeserializeOwned + Send + Display {
    fn fetch_relatives(
        &self,
        filter: T::Filter,
        conn: &mut AsyncPgConnection,
    ) -> impl Future<Output = Result<Vec<T>>> + Send;
}

pub trait Paginate {
    fn paginate(&self) -> Pagination {
        Pagination::default()
    }
}
// If we don't really need pagination (for example, institutions and people),
// you don't have to `impl` the trait for the corresponding filter
impl<T: Paginate> Paginate for Option<T> {
    fn paginate(&self) -> Pagination {
        match self {
            Some(item) => item.paginate(),
            None => Pagination::default(),
        }
    }
}

pub struct Pagination {
    limit: i64,
    offset: i64,
}
impl Default for Pagination {
    fn default() -> Self {
        Pagination {
            limit: 500,
            offset: 0,
        }
    }
}

pub async fn set_transaction_user(user_id: &Uuid, conn: &mut AsyncPgConnection) -> Result<()> {
    diesel::sql_query(format!(r#"set local role "{user_id}""#))
        .execute(conn)
        .await?;

    Ok(())
}