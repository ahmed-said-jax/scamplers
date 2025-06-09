use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use scamplers_core::model::{
    Pagination,
    institution::{
        Institution, InstitutionOrdering, InstitutionQuery, InstitutionSummary, NewInstitution,
    },
};
use scamplers_schema::institution::dsl::{id as id_col, institution, name as name_col};
use uuid::Uuid;

use crate::db::{
    self, AsDieselFilter, AsDieselQueryBase, BoxedDieselExpression, FetchById, FetchByQuery,
    NewBoxedDieselExpression, Write, util::AsIlike,
};

impl Write for NewInstitution {
    type Returns = Institution;
    async fn write(
        self,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> crate::db::error::Result<Self::Returns> {
        let inserted = diesel::insert_into(institution)
            .values(self)
            .returning(Institution::as_returning())
            .get_result(db_conn)
            .await?;

        Ok(inserted)
    }
}

impl AsDieselQueryBase for Institution {
    type QueryBase = institution;

    fn as_diesel_query_base() -> Self::QueryBase {
        institution
    }
}

impl FetchById for Institution {
    type Id = Uuid;
    async fn fetch_by_id(
        id: &Self::Id,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> crate::db::error::Result<Self> {
        let query_base = Self::as_diesel_query_base();
        Ok(query_base
            .find(id)
            .select(Institution::as_select())
            .get_result(db_conn)
            .await?)
    }
}

impl<QuerySource> AsDieselFilter<QuerySource> for InstitutionQuery
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
            query = query.with_condition(id_col.eq_any(ids));
        }

        if let Some(name) = name {
            query = query.with_condition(name_col.ilike(name.as_ilike()));
        }

        query.build()
    }
}

impl FetchByQuery for InstitutionSummary {
    type QueryParams = InstitutionQuery;
    async fn fetch_by_query(
        query: &Self::QueryParams,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> db::error::Result<Vec<Self>> {
        use scamplers_core::model::institution::InstitutionOrdinalColumn::Name;

        let InstitutionQuery {
            order_by,
            pagination: Pagination { limit, offset },
            ..
        } = query;

        let mut statement = Institution::as_diesel_query_base()
            .select(Self::as_select())
            .limit(*limit)
            .offset(*offset)
            .into_boxed();

        let query = query.as_diesel_filter();

        if let Some(query) = query {
            statement = statement.filter(query);
        }

        for InstitutionOrdering { column, descending } in order_by {
            statement = match (column, descending) {
                (Name, false) => statement.then_order_by(name_col.asc()),
                (Name, true) => statement.then_order_by(name_col.desc()),
            };
        }

        Ok(statement.load(db_conn).await?)
    }
}
