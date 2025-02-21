use std::fmt::Display;

use diesel::{
    pg::Pg,
    prelude::*,
};
use diesel_async::RunQueryDsl;
use futures::FutureExt;
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valuable::Valuable;

use super::{
    Create, Paginate, Read, ReadRelatives,
    person::{Person, PersonLite},
};
use crate::schema::{lab, lab_membership, person};

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
#[derive(Deserialize, Validate, Insertable, Identifiable, Selectable, Queryable)]
#[diesel(table_name = lab_membership, check_for_backend(Pg), primary_key(lab_id, member_id))]
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

        Lab::fetch_many(LabFilter { ids: lab_ids }, conn).await
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum Lab {
    Lite(LabLite),
    Full(LabFull),
}

#[derive(Serialize)]
struct LabFull {
    #[serde(flatten)]
    inner: LabLite,
    members: Vec<PersonLite>,
}

#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = lab, check_for_backend(Pg))]
struct LabLite {
    #[serde(flatten)]
    #[diesel(embed)]
    stub: LabStub,
    delivery_dir: String,
    #[diesel(embed)]
    pi: PersonLite,
}

#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = lab, check_for_backend(Pg))]
pub(super) struct LabStub {
    id: Uuid,
    name: String,
    link: String,
}

#[derive(Deserialize, Default, Valuable)]
pub struct LabFilter {
    #[valuable(skip)]
    #[serde(default)]
    ids: Vec<Uuid>,
}
impl Paginate for LabFilter {}
impl LabFilter {
    fn as_sql(&self) -> lab::BoxedQuery<'_, Pg> {
        use lab::dsl::id as id_col;
        let Self { ids, .. } = self;

        let mut query = lab::table.into_boxed();

        if !ids.is_empty() {
            query = query.filter(id_col.eq_any(ids));
        }

        query
    }
}

impl Read for Lab {
    type Filter = LabFilter;
    type Id = Uuid;

    async fn fetch_by_id(id: Self::Id, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self> {
        use lab_membership::lab_id;

        let inner = lab::table
            .find(id)
            .inner_join(person::table)
            .select(LabLite::as_select())
            .first(conn)
            .boxed();

        let members = lab_membership::table
            .inner_join(person::table)
            .filter(lab_id.eq(id))
            .select(PersonLite::as_select())
            .load(conn)
            .boxed();

        let (inner, members) = tokio::try_join!(inner, members)?;

        Ok(Self::Full(LabFull { inner, members }))
    }

    async fn fetch_many(filter: Self::Filter, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Vec<Self>> {
        let labs = filter
            .as_sql()
            .inner_join(person::table)
            .select(LabLite::as_select())
            .load(conn)
            .await?;

        Ok(labs.into_iter().map(|l| Self::Lite(l)).collect())
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
        person_filter: <Person as super::Read>::Filter,
        conn: &mut diesel_async::AsyncPgConnection,
    ) -> super::Result<Vec<Person>> {
        use lab_membership::dsl::lab_id as lab_id_col;

        // Extract the lab_id
        let Self(lab_id) = self;

        // This is our base - a `where` condition on person. I'm not entirely sure why we have to do `select <stuff>
        // from person...inner join lab_membership` rather than `select <stuff> from lab_membership inner join
        // person...`
        let query = person_filter.as_sql();

        // Now we join the lab_membership table
        let query = query.inner_join(lab_membership::table);

        // Filter to make sure we only get the lab we want
        let query = query.filter(lab_id_col.eq(lab_id));

        // Select the columns we want and load
        let members = query.select(PersonLite::as_select()).load(conn).await?;

        // Map them into the `Person` enum. There is a reason for this complexity
        Ok(members.into_iter().map(|p| Person::Lite(p)).collect())
    }
}
