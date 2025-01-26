use diesel::{pg::Pg, prelude::*};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::people;
// create orcid type with validation

#[derive(Insertable, Deserialize)]
#[diesel(table_name = people, check_for_backend(Pg))]
struct NewPerson {
    first_name: String,
    last_name: String,
    email: String,
    institution_id: Uuid,
}

pub struct NewPeople(Vec<NewPerson>);
impl NewPeople {
    pub fn create(&self, conn: &mut PgConnection) -> super::Result<Vec<Person>> {
        use diesel::dsl::insert_into;

        use crate::schema::people::dsl::*;

        Ok(insert_into(people)
            .values(&self.0)
            .returning(Person::as_returning())
            .get_results(conn)?)
    }
}

#[derive(Selectable, Queryable, Debug)]
#[diesel(table_name = people, check_for_backend(Pg))]
pub struct Person {
    id: Uuid,
    first_name: String,
    last_name: String,
    email: String,
    orcid: Option<String>,
}

#[cfg(test)]
mod tests {
    use diesel::{Connection, PgConnection, RunQueryDsl};
    use rstest::rstest;
    use uuid::Uuid;

    use super::{NewPeople, NewPerson};
    use crate::{
        db::{institutions::NewInstitutions, test_utils::db_conn, PgPooledConnection},
        schema,
    };

    #[rstest]
    fn show_errors(mut db_conn: PgPooledConnection) {
        db_conn.begin_test_transaction().unwrap();

        let new_institutions = serde_json::json!([{
            "name": "Hogwarts School of Witchcraft and Wizardry",
            "ms_tenant_id": null
        }]);
        let new_institutions: NewInstitutions = serde_json::from_value(new_institutions).unwrap();

        let new_person = NewPerson {
            first_name: "ahmed".to_string(),
            last_name: "said".to_string(),
            email: String::new(),
            institution_id: Uuid::nil(),
        };
        let err = NewPeople(vec![new_person])
            .create(&mut db_conn)
            .unwrap_err();
        println!("{}", serde_json::to_string(&err).unwrap())
    }
}
