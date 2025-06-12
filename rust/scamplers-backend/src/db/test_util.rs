use std::fmt::Debug;

use crate::{
    db::model::FetchByQuery,
    server::{run_migrations, util::DevContainer},
};
use diesel_async::{
    AsyncPgConnection, RunQueryDsl,
    pooled_connection::{
        AsyncDieselConnectionManager,
        deadpool::{Object, Pool},
    },
};
use pretty_assertions::assert_eq;
use rand::seq::IndexedRandom;
use rstest::fixture;
use scamplers_core::model::{
    institution::NewInstitution,
    lab::NewLab,
    person::{NewPerson, Person, PersonQuery, PersonSummary},
};
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::db::model::Write;

pub const N_INSTITUTIONS: usize = 20;
pub const N_PEOPLE: usize = 100;
pub const N_LABS: usize = 25;
pub const N_LAB_MEMBERS: usize = 5;

struct TestState {
    _container: DevContainer,
    db_pool: Pool<AsyncPgConnection>,
}
impl TestState {
    async fn new() -> Self {
        let name = "scamplers-backend_unit_test";
        let container = DevContainer::new(name, false).await.unwrap();

        let db_config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(
            container.db_url().await.unwrap(),
        );
        let db_pool = Pool::builder(db_config).build().unwrap();

        let mut test_state = Self {
            _container: container,
            db_pool,
        };

        test_state.populate().await;

        test_state
    }

    async fn db_conn(&self) -> Object<AsyncPgConnection> {
        self.db_pool.get().await.unwrap()
    }

    async fn populate(&mut self) {
        let db_conn = self.db_conn().await;
        run_migrations(db_conn).await.unwrap();

        let db_conn = &mut self.db_conn().await;

        let mut institutions = Vec::with_capacity(N_INSTITUTIONS);
        for i in 0..N_INSTITUTIONS {
            let new_institution = NewInstitution {
                id: Uuid::now_v7(),
                name: format!("institution{i}"),
            }
            .write(db_conn)
            .await
            .unwrap();

            institutions.push(new_institution);
        }

        let rng = &mut rand::rng();

        let mut people = Vec::with_capacity(N_PEOPLE);
        for i in 0..N_PEOPLE {
            let institution_id = institutions.choose(rng).unwrap().0.reference.id;

            let new_person = NewPerson {
                name: format!("person{i}"),
                email: format!("person{i}@example.com"),
                institution_id,
                ms_user_id: None,
                orcid: None,
                roles: vec![],
            }
            .write(db_conn)
            .await
            .unwrap();

            people.push(new_person);
        }

        let mut labs = Vec::with_capacity(N_LABS);
        for i in 0..N_LABS {
            let id = |p: &Person| p.summary.reference.id;

            let pi_id = people.choose(rng).map(id).unwrap();
            let name = format!("lab{i}");
            // Use `N_LAB_MEMBERS - 1` because we're expecting to add the PI, so using this constant later can be correct
            let member_ids = people
                .choose_multiple(rng, N_LAB_MEMBERS - 1)
                .map(id)
                .collect();

            let new_lab = NewLab {
                name: name.clone(),
                pi_id,
                delivery_dir: format!("{name}_dir"),
                member_ids,
            }
            .write(db_conn)
            .await
            .unwrap();

            labs.push(new_lab);
        }
    }
}

static TEST_STATE: OnceCell<TestState> = OnceCell::const_new();
pub type DbConnection = Object<AsyncPgConnection>;

#[fixture]
pub async fn db_conn() -> DbConnection {
    let test_state = TEST_STATE.get_or_init(TestState::new).await;

    test_state.db_conn().await
}

#[allow(dead_code)]
pub trait TestDbConnection {
    async fn set_user(&mut self, user_id: &Uuid);
    async fn set_random_user(&mut self);
}

impl TestDbConnection for DbConnection {
    async fn set_user(&mut self, user_id: &Uuid) {
        diesel::sql_query(format!(r#"set role "{user_id}""#))
            .execute(self)
            .await
            .unwrap();
    }

    async fn set_random_user(&mut self) {
        #[allow(clippy::get_first)]
        let user_id = PersonSummary::fetch_by_query(&PersonQuery::default(), self)
            .await
            .unwrap()
            .get(0)
            .unwrap()
            .reference
            .id;

        self.set_user(&user_id).await;
    }
}

pub async fn test_query<Record, Value1, Value2>(
    query: Record::QueryParams,
    mut db_conn: DbConnection,
    expected_len: usize,
    comparison_fn: fn(&Record) -> Value1,
    expected: &[(usize, Value2)],
) where
    Record: FetchByQuery,
    Value1: Debug,
    Value2: Debug + PartialEq<Value1>,
{
    let records = Record::fetch_by_query(&query, &mut db_conn).await.unwrap();
    assert_eq!(records.len(), expected_len);

    for (i, expected_val) in expected {
        assert_eq!(
            *expected_val,
            records.get(*i).map(comparison_fn).unwrap(),
            "record {i} had unexpected value"
        );
    }
}
