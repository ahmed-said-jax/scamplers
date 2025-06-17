use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use scamplers_core::model::{
    Pagination,
    institution::{Institution, InstitutionQuery, InstitutionSummary, NewInstitution},
};
use scamplers_schema::institution::dsl::{id as id_col, institution, name as name_col};
use uuid::Uuid;

use crate::{
    db::{
        model::{self, AsDieselQueryBase},
        util::{AsIlike, BoxedDieselExpression, NewBoxedDieselExpression},
    },
    fetch_by_query,
};

impl model::Write for NewInstitution {
    type Returns = Institution;
    async fn write(
        self,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> super::error::Result<Self::Returns> {
        let inserted = diesel::insert_into(institution)
            .values(self)
            .returning(Institution::as_returning())
            .get_result(db_conn)
            .await?;

        Ok(inserted)
    }
}

impl model::AsDieselQueryBase for InstitutionSummary {
    type QueryBase = institution;

    fn as_diesel_query_base() -> Self::QueryBase {
        institution
    }
}

impl model::AsDieselQueryBase for Institution {
    type QueryBase = <InstitutionSummary as model::AsDieselQueryBase>::QueryBase;

    fn as_diesel_query_base() -> Self::QueryBase {
        InstitutionSummary::as_diesel_query_base()
    }
}

impl model::FetchById for Institution {
    type Id = Uuid;

    async fn fetch_by_id(
        id: &Self::Id,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> super::error::Result<Self> {
        let query_base = Self::as_diesel_query_base();
        Ok(query_base
            .find(id)
            .select(Institution::as_select())
            .first(db_conn)
            .await?)
    }
}

impl<QuerySource> model::AsDieselFilter<QuerySource> for InstitutionQuery
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

impl model::FetchByQuery for InstitutionSummary {
    type QueryParams = InstitutionQuery;

    async fn fetch_by_query(
        query: &Self::QueryParams,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> super::error::Result<Vec<Self>> {
        use scamplers_core::model::institution::InstitutionOrdinalColumn::Name;

        fetch_by_query!(query, [(Name, name_col)], db_conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use scamplers_core::model::institution::*;

    use crate::db::test_util::{DbConnection, N_INSTITUTIONS, db_conn, test_query};

    fn comparison_fn(i: &InstitutionSummary) -> String {
        i.name().to_string()
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn default_institution_query(#[future] db_conn: DbConnection) {
        let expected = [(0, "institution0"), (N_INSTITUTIONS - 1, "institution9")];
        test_query(
            InstitutionQuery::default(),
            db_conn,
            N_INSTITUTIONS,
            comparison_fn,
            &expected,
        )
        .await;
    }

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn specific_institution_query(#[future] db_conn: DbConnection) {
        let query = InstitutionQuery {
            name: Some("institution1".to_string()),
            order_by: vec![InstitutionOrdering {
                column: InstitutionOrdinalColumn::Name,
                descending: true,
            }],
            ..Default::default()
        };
        let expected = [(0, "institution19"), (10, "institution1")];

        test_query(query, db_conn, 11, comparison_fn, &expected).await;
    }
}
