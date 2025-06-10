use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use scamplers_core::model::{
    Pagination,
    lab::{LabOrdering, LabQuery, LabSummary},
};
use scamplers_schema::lab::{self, id as id_col, name as name_col};

use crate::db::{
    AsDieselFilter, AsDieselQueryBase, BoxedDieselExpression, FetchByQuery,
    NewBoxedDieselExpression, util::AsIlike,
};

impl<QuerySource> AsDieselFilter<QuerySource> for LabQuery
where
    id_col: SelectableExpression<QuerySource>,
    name_col: SelectableExpression<QuerySource>,
{
    fn as_diesel_filter<'a>(&'a self) -> Option<crate::db::BoxedDieselExpression<'a, QuerySource>>
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

impl AsDieselQueryBase for LabSummary {
    type QueryBase = lab::table;

    fn as_diesel_query_base() -> Self::QueryBase {
        lab::table
    }
}

impl FetchByQuery for LabSummary {
    type QueryParams = LabQuery;

    async fn fetch_by_query(
        query: &Self::QueryParams,
        db_conn: &mut diesel_async::AsyncPgConnection,
    ) -> crate::db::error::Result<Vec<Self>> {
        use scamplers_core::model::lab::LabOrdinalColumn::Name;

        let LabQuery {
            order_by,
            pagination: Pagination { limit, offset },
            ..
        } = &query;

        let query = query.as_diesel_filter();

        let mut statement = Self::as_diesel_query_base()
            .select(Self::as_select())
            .limit(*limit)
            .offset(*offset)
            .into_boxed();

        if let Some(query) = query {
            statement = statement.filter(query);
        }

        for LabOrdering { column, descending } in order_by {
            statement = match (column, descending) {
                (Name, false) => statement.then_order_by(name_col.asc()),
                (Name, true) => statement.then_order_by(name_col.desc()),
            }
        }

        let labs = statement.load(db_conn).await?;

        Ok(labs)
    }
}
