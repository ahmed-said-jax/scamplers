use std::fmt::Debug;

use argon2::{password_hash::{PasswordHasher, SaltString}, Argon2};
use diesel::{deserialize::{FromSql, FromSqlRow}, expression::AsExpression, pg::Pg, serialize::{ToSql, WriteTuple}, sql_types::{self, Record, SqlType}};
use rand::{distr::Alphanumeric, rngs::{OsRng, StdRng}, Rng, SeedableRng, TryRngCore};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, AsExpression)]
#[diesel(sql_type = crate::schema::sql_types::HashedKey)]
#[serde(transparent)]
struct Key(String);
impl Key {
    pub fn new() -> Self {
        let key = [' '; 32];
        let mut rng = StdRng::from_os_rng();
        let key = key.iter().map(|_| rng.sample(Alphanumeric) as char).collect();

        Self(key)
    }

    pub fn hash(&self) -> HashedKey<&str> {
        let Self(key) = self;

        let mut salt = [0u8; 16];
        OsRng.try_fill_bytes(&mut salt).unwrap();

        let salt = SaltString::encode_b64(&salt).unwrap();

        let argon2 = Argon2::default();
        let hash = argon2.hash_password(key.as_bytes(), &salt).unwrap().to_string();
        
        HashedKey { prefix: &key[0..8], hash }
    }
}
impl Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "KEY".fmt(f)
    }
}

#[derive(AsExpression, Debug, FromSqlRow)]
#[diesel(sql_type = crate::schema::sql_types::HashedKey, postgres_type(name = "hashed_key"))]
struct HashedKey<Str: AsExpression<diesel::sql_types::Text>> where for <'a> &'a Str: AsExpression<diesel::sql_types::Text> {
    prefix: Str,
    hash: String
}

impl ToSql<crate::schema::sql_types::HashedKey, Pg> for HashedKey<&str> {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        let Self { prefix, hash } = self;

        WriteTuple::<(sql_types::Text, sql_types::Text)>::write_tuple(&(prefix, hash), &mut out.reborrow())
    }
}

impl FromSql<crate::schema::sql_types::HashedKey, Pg> for HashedKey<String> {
    fn from_sql(bytes: <Pg as diesel::backend::Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let (prefix, hash) = FromSql::<Record<(sql_types::Text, sql_types::Text)>, Pg>::from_sql(bytes)?;

        Ok(Self{prefix, hash})
    }
}

impl ToSql<crate::schema::sql_types::HashedKey, Pg> for Key {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        let hashed = self.hash();
        <HashedKey<&str> as ToSql<crate::schema::sql_types::HashedKey, Pg>>::to_sql(&hashed, &mut out.reborrow())
    }

}


// enum User {
//     Web {
//         user_id: Uuid,
//         first_name: String,
//         roles: Vec<UserRole>,
//     },
//     Api {
//         user_id: Uuid,
//     },
// }

// impl User {
//     fn id(&self) -> &Uuid {
//         match self {
//             User::Web { user_id, .. } | User::Api { user_id, .. } => user_id,
//         }
//     }

//     async fn fetch_by_api_key(api_key: &Uuid, salt_string: &str, conn: &mut AsyncPgConnection) -> db::Result<Self> {
//         use crate::schema::person::dsl::*;

//         let hash = api_key.hash(salt_string);

//         let user_id = person.filter(api_key_hash.eq(hash)).select(id).first(conn).await?;

//         Ok(Self::Api { user_id })
//     }

//     async fn fetch_by_session_id(
//         session_id: &Uuid,
//         salt_string: &str,
//         conn: &mut AsyncPgConnection,
//     ) -> db::Result<Self> {
//         use crate::schema::{
//             person::dsl::{id as person_id, name as person_name, person},
//             session::dsl::{id_hash, session},
//         };

//         let hash = session_id.hash(salt_string);

//         let (user_id, first_name): (Uuid, String) = session
//             .inner_join(person)
//             .filter(id_hash.eq(hash))
//             .select((person_id, person_name))
//             .first(conn)
//             .await?;
//         // let roles = get_user_roles(user_id.to_string()).execute(conn).await?;

//         Ok(Self::Web {
//             user_id,
//             first_name,
//             roles: Default::default(),
//         })
//     }
// }
