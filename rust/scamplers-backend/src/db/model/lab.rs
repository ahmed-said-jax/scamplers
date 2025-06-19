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
    IsUpdate, Pagination,
    lab::{Lab, LabCore, LabQuery, LabSummary, LabUpdate, NewLab},
    person::PersonSummary,
};
use scamplers_schema::{
    lab::{self, id as id_col, name as name_col, pi_id as pi_id_col},
    lab_membership::{self, lab_id as lab_id_col, member_id as member_id_col},
    person,
};
use uuid::Uuid;

impl model::Write for LabUpdate {
    type Returns = Lab;

    async fn write(
        self,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> super::error::Result<Self::Returns> {
        let (core, add_members, remove_members) =
            (self.core(), self.add_members(), self.remove_members());

        if core.is_update() {
            diesel::update(&core).set(core).execute(db_conn).await?;
        }

        let member_additions: Vec<_> = add_members
            .iter()
            .map(|m_id| (lab_id_col.eq(core.id()), member_id_col.eq(m_id)))
            .collect();

        diesel::insert_into(lab_membership::table)
            .values(member_additions)
            .on_conflict_do_nothing()
            .execute(db_conn)
            .await?;

        let mut member_removal_filter = BoxedDieselExpression::new_expression();
        for member_id in remove_members {
            member_removal_filter = member_removal_filter
                .or_condition(lab_id_col.eq(core.id()).and(member_id_col.eq(member_id)));
        }

        let member_removal_filter = member_removal_filter.build();
        if let Some(f) = member_removal_filter {
            diesel::delete(lab_membership::table)
                .filter(f)
                .execute(db_conn)
                .await?;
        }

        Lab::fetch_by_id(core.id(), db_conn).await
    }
}

impl model::Write for NewLab {
    type Returns = Lab;

    async fn write(
        self,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> super::error::Result<Self::Returns> {
        let (id, pi_id) = diesel::insert_into(lab::table)
            .values(&self)
            .returning((id_col, pi_id_col))
            .get_result(db_conn)
            .await?;

        let mut update = LabUpdate::builder().id(id);

        update = update.add_member(pi_id);
        for member_id in self.member_ids() {
            update = update.add_member(*member_id);
        }

        update.build().write(db_conn).await
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

impl AsDieselQueryBase for LabCore {
    type QueryBase = InnerJoin<lab::table, person::table>;
    fn as_diesel_query_base() -> Self::QueryBase {
        LabSummary::as_diesel_query_base().inner_join(person::table)
    }
}

impl model::FetchById for LabCore {
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

impl model::FetchById for Lab {
    type Id = Uuid;
    async fn fetch_by_id(
        id: &Self::Id,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> crate::db::error::Result<Self> {
        let core = LabCore::fetch_by_id(id, db_conn).await?;
        let members = lab::table::fetch_relatives(id, db_conn).await?;

        Ok(Self::builder().core(core).members(members).build())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use diesel_async::{AsyncConnection, scoped_futures::ScopedFutureExt};
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use scamplers_core::model::{
        lab::{LabQuery, LabSummary, LabUpdate, NewLab},
        person::{PersonQuery, PersonSummary},
    };
    use scamplers_schema::lab;

    use crate::db::{
        model::{FetchByQuery, FetchRelatives, Write},
        test_util::{DbConnection, N_LAB_MEMBERS, N_LABS, db_conn, test_query},
    };

    fn comparison_fn(l: &LabSummary) -> String {
        l.name().to_string()
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn default_query(#[future] db_conn: DbConnection) {
        let expected = [(0, "lab0"), (N_LABS - 1, "lab9")];
        test_query(
            LabQuery::default(),
            db_conn,
            N_LABS,
            comparison_fn,
            &expected,
        )
        .await;
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn new_lab_without_members(#[future] mut db_conn: DbConnection) {
        db_conn
            .test_transaction::<_, crate::db::error::Error, _>(|tx| {
                async move {
                    let pi = PersonSummary::fetch_by_query(&PersonQuery::default(), tx)
                        .await
                        .unwrap()
                        .remove(0);

                    let new_lab = NewLab::builder()
                        .name("Rick Sanchez Lab")
                        .pi_id(*pi.id())
                        .delivery_dir("rick_sanchez")
                        .build();

                    let new_lab = new_lab.write(tx).await.unwrap();

                    // We expect one member - the PI
                    assert_eq!(new_lab.members().len(), 1);
                    assert_eq!(new_lab.members()[0].id(), pi.id());

                    Ok(())
                }
                .scope_boxed()
            })
            .await;
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn remove_lab_members(#[future] mut db_conn: DbConnection) {
        db_conn
            .test_transaction::<_, crate::db::error::Error, _>(|tx| {
                async move {
                    let lab = LabSummary::fetch_by_query(&LabQuery::default(), tx)
                        .await
                        .unwrap()
                        .swap_remove(1);

                    let original_members = lab::table::fetch_relatives(lab.id(), tx).await.unwrap();
                    assert_eq!(original_members.len(), N_LAB_MEMBERS);

                    let member_to_be_removed = original_members[0].id();

                    let updated_lab = LabUpdate::builder()
                        .id(*lab.id())
                        .remove_member(*member_to_be_removed)
                        .build()
                        .write(tx)
                        .await
                        .unwrap();

                    assert_eq!(updated_lab.id(), lab.id());
                    assert_eq!(updated_lab.members().len(), N_LAB_MEMBERS - 1);

                    let extract_ids = |people: &[PersonSummary]| {
                        people.iter().map(|p| *p.id()).collect::<HashSet<_>>()
                    };

                    assert_eq!(
                        extract_ids(&original_members[1..5]),
                        extract_ids(updated_lab.members())
                    );

                    Ok(())
                }
                .scope_boxed()
            })
            .await;
    }
}
