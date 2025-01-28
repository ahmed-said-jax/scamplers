// use diesel::{
//     r2d2::{ConnectionManager, Pool, PooledConnection},
//     PgConnection,
// };
// use diesel_async::{pooled_connection::AsyncDieselConnectionManager,
// AsyncPgConnection}; use diesel_migrations::{embed_migrations,
// EmbeddedMigrations}; use serde::Serialize;

use std::str::FromStr;

use diesel::result::DatabaseErrorInformation;
use diesel_async::AsyncPgConnection;
use futures::FutureExt;
use itertools::Itertools;
use regex::Regex;
use serde::Serialize;
use valuable::Valuable;

// pub mod cdna;
// pub mod chemistries;
// pub mod chromium_datasets;
// pub mod chromium_libraries;
// pub mod chromium_runs;
// pub mod dataset_metadata;
// pub mod index_sets;
pub mod institution;

// the following traits are not used to enforce anything, they just help to
// provide a uniform interface for database CRUD operations

// we don't need to `impl<T> Create for Vec<T>` for generic `T` because diesel
// allows us to insert multiple records at once, so we can just `impl Create`
// for a concrete Vec<T>
pub trait Create {
    type Returns;

    async fn create(&self, conn: &mut AsyncPgConnection) -> Result<Self::Returns>;
}

pub trait Update {
    type Returns;

    async fn update(&self, conn: &mut AsyncPgConnection) -> Result<Self::Returns>;
}

pub trait Upsert {
    type Returns;
    async fn upsert(&self, conn: &mut AsyncPgConnection) -> Result<Self::Returns>;
}

impl<T: Upsert> Upsert for Vec<T> {
    type Returns = Vec<T::Returns>;

    async fn upsert(&self, conn: &mut AsyncPgConnection) -> Result<Self::Returns> {
        let mut updated = Vec::with_capacity(self.len());

        for record in self {
            updated.push(record.upsert(conn).await?);
        }

        Ok(updated)
    }
}

#[derive(Debug, Serialize, Valuable, strum::EnumString, Default, strum::Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
enum PublicEntity {
    Institution,
    Person,
    Lab,
    Sample,
    Library,
    SequencingRun,
    Dataset,
    #[default]
    Unknown
}

#[derive(thiserror::Error, Debug, Valuable, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Error {
    #[error("duplicate record")]
    DuplicateRecord{entity: PublicEntity, field: Option<String>, value: Option<String>},
    #[error("referenced record not found")]
    ReferenceNotFound{entity: PublicEntity, referenced_entity: PublicEntity, value: Option<String>},
    #[error("record not found")]
    RecordNotFound,
    #[error("other error")]
    Other{message: String}
}
type Result<T> = std::result::Result<T, Error>;

impl Error {
    fn from_other_error(err: impl std::error::Error) -> Self {
        Self::Other{message: format!("{err:?}")}
    }
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        use diesel::result::Error::*;
        match err {
            DatabaseError(kind, info) => Self::from((kind, info)),
            NotFound => Self::RecordNotFound,
            _ => Self::from_other_error(err)
        }
    }
}

impl From<(diesel::result::DatabaseErrorKind, Box<dyn DatabaseErrorInformation + Send + Sync>)> for Error {
    fn from((kind, info): (diesel::result::DatabaseErrorKind, Box<dyn DatabaseErrorInformation + Send + Sync>)) -> Self {
        use diesel::result::DatabaseErrorKind::*;

        let entity = PublicEntity::from_str(info.table_name().unwrap_or_default()).unwrap_or_default();

        let field = info.column_name().map(str::to_string);

        let detail_regex = Regex::new(r#"Key \(.*\)=(\(.*\)).*"#).unwrap();
        let details = info.details().unwrap_or_default();
        let value = detail_regex.captures(details).and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()));

        match kind {
            UniqueViolation => Self::DuplicateRecord {entity, field, value},
            ForeignKeyViolation => {
                let referenced_entity = details.split_whitespace().last().unwrap_or_default().replace("\"", "");
                let referenced_entity = referenced_entity.strip_suffix(".").unwrap_or_default();
                let referenced_entity = PublicEntity::from_str(referenced_entity).unwrap_or_default();

                Self::ReferenceNotFound { entity, referenced_entity, value }
            },
            _ => Self::from_other_error(diesel::result::Error::DatabaseError(kind, info))
        }
    }

}
// pub mod labs;
// pub mod measurements;
// pub mod multiplexed_suspensions;
// pub mod people;
// pub mod sample_metadata;
// pub mod sequencing_runs;
// pub mod specimens;
// pub mod suspensions;

// write error tests to make sure they look right

// #[derive(Debug, thiserror::Error, Serialize)]
// #[serde(tag = "type", rename_all = "snake_case")]
// pub enum Error {
//     #[error("{message}")]
//     DuplicateRecord { message: String },
//     #[error("{message}")]
//     RecordNotFound { message: String },
//     #[error("{0}")]
//     Other(String),
// }

// impl From<diesel::result::Error> for Error {
//     fn from(err: diesel::result::Error) -> Self {
//         use diesel::result::{DatabaseErrorKind::*, Error::*};

//         let DatabaseError(err_kind, err_info) = &err else {
//             return Self::from_other_error(err);
//         };
//         let message = err_info
//             .details()
//             .unwrap_or_default()
//             .to_string()
//             .replace("\"", "\'")
//             .replace("table ", "");

//         match err_kind {
//             UniqueViolation => Self::DuplicateRecord { message },
//             ForeignKeyViolation => Self::RecordNotFound { message },
//             _ => Self::from_other_error(err),
//         }
//     }
// }

// type Result<T> = std::result::Result<T, Error>;

// #[cfg(test)]
// mod test_utils {
//     use std::env;

//     use diesel::r2d2::{ConnectionManager, Pool};
//     use diesel_migrations::MigrationHarness;
//     use rstest::fixture;
//     use testcontainers_modules::{
//         postgres::Postgres as PostgresImage,
//         testcontainers::{core::ExecCommand, runners::SyncRunner, Container,
// ImageExt},     };

//     use super::{PgConnectionManager, PgConnectionPool, PgPooledConnection,
// MIGRATIONS};

//     #[fixture]
//     #[once]
//     fn container() -> Container<PostgresImage> {
//         let postgres_version =
// env::var("POSTGRES_VERSION").unwrap_or("latest".to_string());

//         PostgresImage::default()
//             .with_host_auth()
//             .with_tag(&postgres_version)
//             .start()
//             .unwrap()
//     }

//     #[fixture]
//     #[once]
//     fn db_conn_pool(container: &Container<PostgresImage>) -> PgConnectionPool
// {         let host = container.get_host().unwrap().to_string();

//         let dbname = "scamplers-test";
//         let username = "postgres";

//         let cmd = ExecCommand::new([
//             "createdb",
//             &dbname,
//             "--username",
//             username,
//             "--host",
//             &host,
//             "--port",
//             "5432",
//         ]);

//         let port = container.get_host_port_ipv4(5432).unwrap();

//         container
//             .exec(cmd)
//             .unwrap()
//             .stdout()
//             .read_to_end(&mut Vec::new())
//             .unwrap();

//         let manager: PgConnectionManager =
//             
// ConnectionManager::new(format!("postgres://{username}@{host}:{port}/{dbname}"
// ));

//         let pool = Pool::builder().build(manager).unwrap();

//         let mut conn = pool.get().unwrap();
//         conn.run_pending_migrations(MIGRATIONS).unwrap();

//         pool
//     }

//     #[fixture]
//     pub fn db_conn(db_conn_pool: &PgConnectionPool) -> PgPooledConnection {
//         db_conn_pool.get().unwrap()
//     }
// }
