use diesel::{pg::Pg, prelude::*};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::institutions;

#[derive(Clone, Insertable, Deserialize)]
#[diesel(table_name = institutions, check_for_backend(Pg))]
struct NewInstitution {
    name: String,
    ms_tenant_id: Option<Uuid>,
}

#[derive(Deserialize)]
#[serde(transparent)]
pub struct NewInstitutions(Vec<NewInstitution>);
impl NewInstitutions {
    pub fn create(&self, conn: &mut PgConnection) -> super::Result<Vec<Institution>> {
        use diesel::dsl::insert_into;

        use crate::schema::institutions::dsl::*;

        Ok(insert_into(institutions)
            .values(&self.0)
            .returning(Institution::as_returning())
            .get_results(conn)?)
    }
}

#[derive(AsChangeset, Deserialize, Identifiable)]
#[diesel(table_name = institutions)]
#[diesel(check_for_backend(Pg))]
pub struct InstitutionUpdate {
    id: Uuid,
    name: Option<String>,
    ms_tenant_id: Option<Uuid>,
}
impl InstitutionUpdate {
    async fn update(&self, conn: &mut PgConnection) -> super::Result<Institution> {
        use diesel::dsl::update;

        Ok(update(self)
            .set(self)
            .returning(Institution::as_returning())
            .get_result(conn)?)
    }
}

#[derive(Serialize, Selectable, Queryable, Debug)]
#[diesel(check_for_backend(Pg))]
pub struct Institution {
    id: Uuid,
    name: String,
}

pub struct InstitutionFilter {
    name: Option<String>,
}

impl Institution {
    pub async fn find(
        filter: &InstitutionFilter,
        conn: &mut PgConnection,
    ) -> super::Result<Vec<Self>> {
        use crate::schema::institutions::dsl::*;

        let mut query = institutions.into_boxed().select(Self::as_select());

        if let Some(institution_name) = &filter.name {
            query = query.filter(name.ilike(format!("%{institution_name}%")));
        }

        Ok(query.get_results(conn)?)
    }
}

#[cfg(test)]
mod tests {
    use diesel::{Connection, PgConnection};
    use rstest::rstest;

    use super::{NewInstitution, NewInstitutions};
    use crate::db::{test_utils::db_conn, PgPooledConnection};

    #[rstest]
    fn show_errors(mut db_conn: PgPooledConnection) {
        db_conn.begin_test_transaction().unwrap();

        let new_institution = NewInstitution {
            name: "Hogwarts School of Witchcraft and Wizardry".to_string(),
            ms_tenant_id: None,
        };
        let duplicate_institution = new_institution.clone();

        let new_institutions = NewInstitutions(vec![new_institution, duplicate_institution]);
        let err = new_institutions.create(&mut db_conn).unwrap_err();
        println!("{}", serde_json::to_string(&err).unwrap())
    }
}
