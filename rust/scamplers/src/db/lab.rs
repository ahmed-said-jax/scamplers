use camino::Utf8PathBuf;
use diesel::{alias, pg::Pg, prelude::*, BelongingToDsl};
use diesel_async::RunQueryDsl;
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valuable::Valuable;
use crate::schema::{lab, lab_membership, person};

use super::{person::PersonLite, Create, Paginate, Read};

// This is the first instance where one API body might represent multiple queries. You'll find a top-level struct that represents the whole API request (here, it's `NewLab`), and then sub-structs that hold references. These sub-structs represent the individual queries that make up a whole creation or update event.

#[derive(Deserialize, Validate, Insertable)]
#[garde(allow_unvalidated)]
#[diesel(table_name = lab, check_for_backend(Pg))]
pub struct NewLab {
    name: String,
    pi_id: Uuid,
    delivery_dir: String,
    #[diesel(skip_insertion)]
    member_ids: Vec<Uuid>
}

// In theory, we should use a struct like `LabMembership<U>`, where U satisfies some trait bounds such that it can be a &Uuid or a Uuid. That way, we can reuse this struct as part of creating a new lab, or as its own query to update a lab
#[derive(Deserialize, Validate, Insertable, Identifiable, Selectable, Queryable, Associations)]
#[diesel(table_name = lab_membership, check_for_backend(Pg), belongs_to(LabRow, foreign_key = lab_id), belongs_to(PersonLite, foreign_key = member_id), primary_key(lab_id, member_id))]
#[garde(allow_unvalidated)]
struct LabMembership {
    lab_id: Uuid,
    member_id: Uuid
}

impl Create for Vec<NewLab> {
    type Returns = Vec<Lab>;

    async fn create(
            &self,
            conn: &mut diesel_async::AsyncPgConnection,
        ) -> super::Result<Self::Returns> {
            use lab::id;


            let inserted_lab_ids: Vec<Uuid> = diesel::insert_into(lab::table).values(self).returning(id).get_results(conn).await?;

            // This is kind of disgustingly complicated. Maybe we should implment `Create` for `NewLab` rather than `Vec<NewLab>`
            let membership_insertions: Vec<Vec<LabMembership>> = inserted_lab_ids.iter().zip(self).map(|(lab_id, lab_data)| lab_data.member_ids.iter().map(|member_id| LabMembership {lab_id: *lab_id, member_id: *member_id} ).collect()).collect();

            // TODO: we may want to change this to `futures::future::try_join_all` for query pipelining
            for memberships in membership_insertions {
                diesel::insert_into(lab_membership::table).values(memberships).execute(conn).await?;
            }

            let lab_rows: Vec<LabRow> = lab::table.inner_join(person::table).select(LabRow::as_select()).filter(id.eq_any(inserted_lab_ids)).load(conn).await?;

            let labs: Vec<(LabMembership, PersonLite)> = LabMembership::belonging_to(&lab_rows).inner_join(person::table).select((LabMembership::as_select(), PersonLite::as_select())).load(conn).await?;

            let labs = labs.grouped_by(&lab_rows).into_iter().zip(lab_rows).map(|(p, lab)| Lab{lab, members: p.into_iter().map(|(_, member)| member).collect()}).collect();

            Ok(labs)
    }
}
#[derive(Serialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = lab, check_for_backend(Pg))]
pub struct LabRow {
    id: Uuid,
    name: String,
    delivery_dir: String,
    link: String,
    #[diesel(embed)]
    pi: PersonLite
}

// This excludes the `email` and `orcid` fields from the full `PersonRow`. That's for the sake of keeping responses human-readable, and it's probably the minimum information someone using a front-end might want to see. But we can just use `PersonRow` if we want. I don't suggest using the full `Person` struct, as that contains an additional layer of nesting by getting the person's institution
#[derive(Serialize)]
pub struct Lab {
    #[serde(flatten)]
    lab: LabRow,
    members: Vec<PersonLite>
}

impl Lab {
    #[diesel::dsl::auto_type(no_type_alias)]
    fn base_query() -> _ {
        lab::table.inner_join(person::table)
    }
}

#[derive(Deserialize, Default, Valuable)]
pub struct LabFilter {
    #[valuable(skip)]
    #[serde(default)]
    ids: Vec<Uuid>
}
impl Paginate for LabFilter {}

impl Read for Lab {
    type Id = Uuid;
    type Filter = LabFilter;

    async fn fetch_by_id(
            id: Self::Id,
            conn: &mut diesel_async::AsyncPgConnection,
        ) -> super::Result<Self> {
        let lab = Self::base_query().select(LabRow::as_select()).filter(lab::id.eq(id)).first(conn).await?;

        let members = LabMembership::belonging_to(&lab).inner_join(person::table).select(PersonLite::as_select()).load(conn).await?;

        Ok(Lab {
            lab,
            members
        })
    }

    async fn fetch_many(
            filter: Self::Filter,
            conn: &mut diesel_async::AsyncPgConnection,
        ) -> super::Result<Vec<Self>> {
        todo!()
    }
}