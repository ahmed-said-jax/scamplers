use std::str::FromStr;
use diesel::{backend::Backend, deserialize::{FromSql, FromSqlRow}, expression::AsExpression, pg::Pg, prelude::*, serialize::ToSql, sql_types::{self, SqlType}};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::{self, SessionIdOrApiKey};

#[derive(
    Clone,
    SqlType,
    FromSqlRow,
    strum::VariantArray,
    AsExpression,
    Debug,
    strum::IntoStaticStr,
    strum::EnumString,
    PartialEq,
    Deserialize,
    Serialize,
)]
#[strum(serialize_all = "snake_case")]
#[diesel(sql_type = sql_types::Text)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    AppAdmin,
    ComputationalStaff,
    LabStaff,
}

impl FromSql<sql_types::Text, Pg> for UserRole {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let raw: String = FromSql::<sql_types::Text, diesel::pg::Pg>::from_sql(bytes)?;
        Ok(Self::from_str(&raw).unwrap())
    }
}

impl ToSql<sql_types::Text, diesel::pg::Pg> for UserRole {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        let as_str: &str = self.into();
        ToSql::<sql_types::Text, Pg>::to_sql(as_str, out)
    }
}

// This represents a person as a user of the application, which is useful to send across to the front-end for neat dynamic formatting
pub enum User {
    Web {
        user_id: Uuid,
        first_name: String,
        roles: Vec<UserRole>,
    },
    Api {
        user_id: Uuid,
    },
}

impl User {
    pub fn id(&self) -> &Uuid {
        match self {
            User::Web { user_id, .. } | User::Api { user_id, .. } => user_id,
        }
    }

    pub async fn fetch_by_api_key(api_key: &Uuid, conn: &mut AsyncPgConnection) -> db::Result<Self> {
        use crate::schema::person::dsl::*;

        let hash = api_key.hash();

        let user_id = person
            .filter(api_key_hash.eq(hash))
            .select(id)
            .first(conn)
            .await?;

        Ok(Self::Api { user_id })
    }

    pub async fn fetch_by_session_id(
        session_id: &Uuid,
        conn: &mut AsyncPgConnection,
    ) -> db::Result<Self> {
        use crate::schema::{
            cache::dsl::{cache, session_id_hash},
            person::dsl::{first_name as person_first_name, id as person_id, person},
        };

        let hash = session_id.hash();

        let (user_id, user_first_name) = cache
            .inner_join(person)
            .filter(session_id_hash.eq(hash))
            .select((person_id, person_first_name))
            .first(conn)
            .await?;
        let roles = Vec::with_capacity(0); // TODO: actually get user_roles

        Ok(Self::Web {
            user_id,
            first_name: user_first_name,
            roles,
        })
    }
}
