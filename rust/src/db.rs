// use diesel::{
//     r2d2::{ConnectionManager, Pool, PooledConnection},
//     PgConnection,
// };
// use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
// use diesel_migrations::{embed_migrations, EmbeddedMigrations};
// use serde::Serialize;

use diesel_async::{AsyncConnection, AsyncPgConnection};
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

// the following traits are not used to enforce anything, they just help to provide a uniform interface for database CRUD operations
pub trait Create {
    type Returns;

    async fn create(&mut self, conn: &mut AsyncPgConnection) -> Result<Self::Returns>;
}

pub trait Update {
    type Returns;

    async fn update(&mut self, conn: &mut AsyncPgConnection) -> Result<Self::Returns>;
}

pub trait Upsert {
    type Returns;

    async fn upsert(&mut self, conn: &mut AsyncPgConnection) -> Result<Self::Returns>;
}

pub struct UpsertMany<T: Upsert>(pub Vec<T>);
impl<T: Upsert>Upsert for UpsertMany<T> {
    type Returns = Vec<T::Returns>;

    // can we do something a bit more slick here?
    async fn upsert(&mut self, conn: &mut AsyncPgConnection) -> Result<Self::Returns> {
        let mut updated = Vec::with_capacity(self.0.len());

        for item in &mut self.0 {
            let new = item.upsert(conn).await?;
            updated.push(new);
        }

        Ok(updated)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),
}
type Result<T> = std::result::Result<T, Error>;
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

// impl Error {
//     fn from_other_error(err: impl std::error::Error) -> Self {
//         Self::Other(format!("{err:#}"))
//     }
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
//         testcontainers::{core::ExecCommand, runners::SyncRunner, Container, ImageExt},
//     };

//     use super::{PgConnectionManager, PgConnectionPool, PgPooledConnection, MIGRATIONS};

//     #[fixture]
//     #[once]
//     fn container() -> Container<PostgresImage> {
//         let postgres_version = env::var("POSTGRES_VERSION").unwrap_or("latest".to_string());

//         PostgresImage::default()
//             .with_host_auth()
//             .with_tag(&postgres_version)
//             .start()
//             .unwrap()
//     }

//     #[fixture]
//     #[once]
//     fn db_conn_pool(container: &Container<PostgresImage>) -> PgConnectionPool {
//         let host = container.get_host().unwrap().to_string();

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
//             ConnectionManager::new(format!("postgres://{username}@{host}:{port}/{dbname}"));

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
