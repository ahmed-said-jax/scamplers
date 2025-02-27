use std::fmt::Display;

use diesel::{BelongingToDsl, dsl::InnerJoin, pg::Pg, prelude::*};
use diesel_async::RunQueryDsl;
use futures::FutureExt;
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valuable::Valuable;

use super::{AsDieselExpression, Create, BoxedDieselExpression, Read, ReadRelatives, person::Person};
use crate::schema::{institution, lab, lab_membership, person};

// This is the first instance where one API body might represent multiple
// queries. You'll find a top-level struct that represents the whole API request
// (here, it's `NewLab`), and then sub-structs that hold references. These
// sub-structs represent the individual queries that make up a whole creation or
// update event.
#[derive(Deserialize, Validate, Insertable, Valuable)]
#[garde(allow_unvalidated)]
#[diesel(table_name = lab, check_for_backend(Pg))]
pub struct NewLab {
    #[garde(length(min = 1))]
    name: String,
    #[valuable(skip)]
    pi_id: Uuid,
    delivery_dir: String,
    #[diesel(skip_insertion)]
    #[valuable(skip)]
    member_ids: Vec<Uuid>,
}

impl Create for Vec<NewLab> {
    type Returns = Vec<Lab>;

    async fn create(&self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
        use lab::id;

        let new_lab_ids: Vec<Uuid> = diesel::insert_into(lab::table)
            .values(self)
            .returning(id)
            .get_results(conn)
            .await?;

        let n_members = self.iter().map(|NewLab { member_ids, .. }| member_ids.len() + 1).sum(); // Add 1 so we can add the PI

        let mut member_insertions = Vec::with_capacity(n_members);
        for (lab_id, NewLab { member_ids, pi_id, .. }) in new_lab_ids.iter().zip(self) {
            let this_lab_member_insertions = member_ids.iter().map(|member_id| LabMembership {
                lab_id: *lab_id,
                member_id: *member_id,
            });

            member_insertions.extend(this_lab_member_insertions);

            // Add the PI just in case
            member_insertions.push(LabMembership {
                lab_id: *lab_id,
                member_id: *pi_id,
            });
        }

        // We take advantage of the fact that adding lab members returns the `Lab` because that is probably desirable
        // for an API
        let labs = member_insertions.create(conn).await?;

        Ok(labs)
    }
}

// In theory, we should use a struct like `LabMembership<U>`, where U satisfies
// some trait bounds such that it can be a &Uuid or a Uuid. That way, we can
// reuse this struct as part of creating a new lab, or as its own query to
// update a lab. However, UUIDs are 16 bytes - very cheap to copy by value, so
// it's not worth it.
#[derive(Deserialize, Validate, Insertable, Identifiable, Selectable, Queryable, Associations)]
#[diesel(table_name = lab_membership, check_for_backend(Pg), primary_key(lab_id, member_id), belongs_to(LabInner, foreign_key = lab_id), belongs_to(Person, foreign_key = member_id))]
#[garde(allow_unvalidated)]
struct LabMembership {
    lab_id: Uuid,
    member_id: Uuid,
}

impl Create for Vec<LabMembership> {
    type Returns = Vec<Lab>;

    async fn create(&self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
        use lab_membership::lab_id;

        let lab_ids = diesel::insert_into(lab_membership::table)
            .values(self)
            .on_conflict_do_nothing()
            .returning(lab_id)
            .get_results(conn)
            .await?;

        Lab::fetch_many(LabQuery { ids: lab_ids }, conn).await
    }
}

#[derive(Serialize)]
pub struct Lab {
    #[serde(flatten)]
    inner: LabInner,
    members: Vec<Person>,
}

#[derive(Serialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = lab, check_for_backend(Pg))]
struct LabInner {
    #[serde(skip)]
    id: Uuid,
    #[serde(flatten)]
    #[diesel(embed)]
    stub: LabStub,
    delivery_dir: String,
    #[diesel(embed)]
    pi: Person,
}

#[derive(Serialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = lab, check_for_backend(Pg))]
pub(super) struct LabStub {
    id: Uuid,
    name: String,
    link: String,
}

#[derive(Deserialize, Default, Valuable)]
pub struct LabQuery {
    #[valuable(skip)]
    #[serde(default)]
    ids: Vec<Uuid>,
}

impl <T> AsDieselExpression<T> for LabQuery where lab::id: SelectableExpression<T> {
    fn as_diesel_expression<'a>(&'a self) -> Option<BoxedDieselExpression<'a, T>> where T: 'a {
        use lab::dsl::id as id_col;

        let Self { ids } = self;

        if matches!((ids.is_empty(),), (true,)) {
            return None;
        }

        let mut query: BoxedDieselExpression<T> = Box::new(id_col.is_not_null());

        if !ids.is_empty() {
            query = Box::new(query.and(id_col.eq_any(ids)));
        }

        Some(query)
    }
}

impl Lab {
    fn base_query() -> InnerJoin<lab::table, InnerJoin<person::table, institution::table>> {
        lab::table.inner_join(Person::base_query())
    }
}

impl Read for Lab {
    type QueryParams = LabQuery;
    type Id = Uuid;

    async fn fetch_by_id(id: Self::Id, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self> {
        use lab::id as id_col;
        use lab_membership::lab_id;

        let inner = Self::base_query()
            .filter(id_col.eq(id))
            .select(LabInner::as_select())
            .first(conn)
            .boxed();

        let person_institution = Person::base_query();

        let members = lab_membership::table
            .inner_join(person_institution)
            .filter(lab_id.eq(id))
            .select(Person::as_select())
            .load(conn)
            .boxed();

        let (inner, members) = tokio::try_join!(inner, members)?;

        Ok(Self { inner, members })
    }

    async fn fetch_many(filter: Self::QueryParams, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Vec<Self>> {
        use lab::name as name_col;

        let filter = filter.as_diesel_expression();

        let labs = Lab::base_query().order_by(name_col).select(LabInner::as_select());

        let labs = match filter {
            Some(f) => labs.filter(f).load(conn).await?,
            None => labs.load(conn).await?,
        };

        let members = LabMembership::belonging_to(&labs)
            .inner_join(Person::base_query())
            .select((LabMembership::as_select(), Person::as_select()))
            .load(conn)
            .await?;

        let labs: Vec<Self> = members
            .grouped_by(&labs)
            .into_iter()
            .zip(labs)
            .map(|(p, lab)| Lab {
                inner: lab,
                members: p.into_iter().map(|(_, person)| person).collect(),
            })
            .collect();

        Ok(labs)
    }
}

#[derive(Deserialize)]
#[serde(transparent)]
pub struct LabId(Uuid);

impl Display for LabId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl ReadRelatives<Person> for LabId {
    async fn fetch_relatives(
        &self,
        person_filter: <Person as super::Read>::QueryParams,
        conn: &mut diesel_async::AsyncPgConnection,
    ) -> super::Result<Vec<Person>> {
        use lab_membership::dsl::lab_id as lab_id_col;

        // Extract the lab_id
        let Self(lab_id) = self;

        let query = Person::base_query()
            .inner_join(lab_membership::table)
            .filter(lab_id_col.eq(lab_id))
            .select(Person::as_select());

        let filter = person_filter.as_diesel_expression();

        let members = match filter {
            Some(f) => query.filter(f).load(conn).await?,
            None => query.load(conn).await?,
        };

        Ok(members)
    }
}
