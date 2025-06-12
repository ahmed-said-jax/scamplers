use crate::{
    db::{
        model::{self, AsDieselQueryBase, FetchById, FetchRelatives},
        util::{AsIlike, BoxedDieselExpression, NewBoxedDieselExpression},
    },
    fetch_by_query,
};
use diesel::{dsl::InnerJoin, prelude::*};
use diesel_async::RunQueryDsl;
use scamplers_core::model::{
    Pagination,
    lab::{Lab, LabQuery, LabSummary, LabUpdate, LabUpdateWithMembers, LabWithMembers, NewLab},
    person::PersonSummary,
};
use scamplers_schema::{
    lab::{self, id as id_col, name as name_col},
    lab_membership::{self, lab_id as lab_id_col, member_id as member_id_col},
    person,
};
use uuid::Uuid;

impl model::Write for LabUpdateWithMembers {
    type Returns = LabWithMembers;

    async fn write(
        self,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> super::error::Result<Self::Returns> {
        let Self {
            update,
            add_members,
            remove_members,
        } = self;

        match update {
            LabUpdate {
                name: None,
                pi_id: None,
                delivery_dir: None,
                ..
            } => (),
            _ => {
                diesel::update(&update)
                    .set(&update)
                    .execute(db_conn)
                    .await?;
            }
        }

        let LabUpdate { id: lab_id, .. } = &update;

        let member_additions: Vec<_> = add_members
            .iter()
            .map(|m_id| (lab_id_col.eq(lab_id), member_id_col.eq(m_id)))
            .collect();

        diesel::insert_into(lab_membership::table)
            .values(member_additions)
            .on_conflict_do_nothing()
            .execute(db_conn)
            .await?;

        let mut member_removal_filter = BoxedDieselExpression::new_expression();
        for member_id in remove_members {
            member_removal_filter = member_removal_filter
                .or_condition(lab_id_col.eq(lab_id).and(member_id_col.eq(member_id)));
        }

        let member_removal_filter = member_removal_filter.build();
        if let Some(f) = member_removal_filter {
            diesel::delete(lab_membership::table)
                .filter(f)
                .execute(db_conn)
                .await?;
        }

        LabWithMembers::fetch_by_id(lab_id, db_conn).await
    }
}

impl model::Write for NewLab {
    type Returns = LabWithMembers;

    async fn write(
        mut self,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> super::error::Result<Self::Returns> {
        let id = diesel::insert_into(lab::table)
            .values(&self)
            .returning(id_col)
            .get_result(db_conn)
            .await?;

        self.member_ids.push(self.pi_id);

        let update = LabUpdateWithMembers {
            update: LabUpdate {
                id,
                ..Default::default()
            },
            add_members: self.member_ids,
            ..Default::default()
        };

        update.write(db_conn).await
    }
}

impl<QuerySource> model::AsDieselFilter<QuerySource> for LabQuery
where
    id_col: SelectableExpression<QuerySource>,
    name_col: SelectableExpression<QuerySource>,
{
    fn as_diesel_filter<'a>(&'a self) -> Option<BoxedDieselExpression<'a, QuerySource>>
    where
        QuerySource: 'a,
    {
        let Self { ids, name, .. } = self;

        let mut query = BoxedDieselExpression::new_expression();

        if !ids.is_empty() {
            query = query.and_condition(id_col.eq_any(ids));
        }

        if let Some(name) = name {
            query = query.and_condition(name_col.ilike(name.as_ilike()));
        }

        query.build()
    }
}

impl AsDieselQueryBase for LabSummary {
    type QueryBase = lab::table;

    fn as_diesel_query_base() -> Self::QueryBase {
        lab::table
    }
}

impl model::FetchByQuery for LabSummary {
    type QueryParams = LabQuery;

    async fn fetch_by_query(
        query: &Self::QueryParams,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> crate::db::error::Result<Vec<Self>> {
        use scamplers_core::model::lab::LabOrdinalColumn::Name;

        fetch_by_query!(query, [(Name, name_col)], db_conn)
    }
}

impl AsDieselQueryBase for Lab {
    type QueryBase = InnerJoin<lab::table, person::table>;
    fn as_diesel_query_base() -> Self::QueryBase {
        LabSummary::as_diesel_query_base().inner_join(person::table)
    }
}

impl model::FetchById for Lab {
    type Id = Uuid;

    async fn fetch_by_id(
        id: &Self::Id,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> crate::db::error::Result<Self> {
        Ok(Self::as_diesel_query_base()
            .select(Self::as_select())
            .filter(id_col.eq(id))
            .get_result(db_conn)
            .await?)
    }
}

impl model::FetchRelatives<PersonSummary> for lab::table {
    type Id = Uuid;

    async fn fetch_relatives(
        lab_id: &Self::Id,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> crate::db::error::Result<Vec<PersonSummary>> {
        let members = lab_membership::table
            .filter(lab_id_col.eq(lab_id))
            .inner_join(PersonSummary::as_diesel_query_base())
            .select(PersonSummary::as_select())
            .load(db_conn)
            .await?;

        Ok(members)
    }
}

impl model::FetchById for LabWithMembers {
    type Id = Uuid;
    async fn fetch_by_id(
        id: &Self::Id,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> crate::db::error::Result<Self> {
        let members = lab::table::fetch_relatives(id, db_conn).await?;
        let lab = Lab::fetch_by_id(id, db_conn).await?;

        Ok(Self { lab, members })
    }
}
