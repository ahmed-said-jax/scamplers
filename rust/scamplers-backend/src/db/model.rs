use super::error;
use diesel_async::AsyncPgConnection;

use crate::db::util::{BoxedDieselExpression, NewBoxedDieselExpression};

pub mod chromium;
pub mod dataset_metadata;
pub mod institution;
pub mod lab;
pub mod measurements;
pub mod person;
pub mod sample_metadata;
pub mod sequencing_run;
pub mod specimen;
pub mod units;

trait AsDieselFilter<QuerySource = ()> {
    fn as_diesel_filter<'a>(&'a self) -> Option<BoxedDieselExpression<'a, QuerySource>>
    where
        QuerySource: 'a;
}

impl AsDieselFilter for () {
    fn as_diesel_filter<'a>(&'a self) -> Option<BoxedDieselExpression<'a, ()>>
    where
        (): 'a,
    {
        BoxedDieselExpression::new_expression().build()
    }
}

impl<Query, QuerySource> AsDieselFilter<QuerySource> for Option<Query>
where
    Query: AsDieselFilter<QuerySource>,
{
    fn as_diesel_filter<'a>(&'a self) -> Option<BoxedDieselExpression<'a, QuerySource>>
    where
        QuerySource: 'a,
    {
        match self {
            Some(query) => query.as_diesel_filter(),
            None => BoxedDieselExpression::new_expression().build(),
        }
    }
}

trait AsDieselQueryBase {
    type QueryBase;

    fn as_diesel_query_base() -> Self::QueryBase;
}

pub trait Write {
    type Returns;

    fn write(
        self,
        db_conn: &mut AsyncPgConnection,
    ) -> impl Future<Output = error::Result<Self::Returns>> + Send;
}

pub trait FetchById: Sized {
    type Id;

    fn fetch_by_id(
        id: &Self::Id,
        db_conn: &mut AsyncPgConnection,
    ) -> impl Future<Output = error::Result<Self>> + Send;
}

pub trait FetchByQuery: Sized {
    type QueryParams;

    fn fetch_by_query(
        query: &Self::QueryParams,
        db_conn: &mut AsyncPgConnection,
    ) -> impl Future<Output = error::Result<Vec<Self>>> + Send;
}

pub trait FetchRelatives<R>: diesel::Table {
    type Id;

    fn fetch_relatives(
        id: &Self::Id,
        db_conn: &mut AsyncPgConnection,
    ) -> impl Future<Output = error::Result<Vec<R>>> + Send;
}

#[macro_export]
macro_rules! fetch_by_query {
    ($query:ident, [$(($ordinal_col_enum_variant:ident, $corresponding_db_col:ident)),*], $db_conn:ident) => {{
        use super::AsDieselFilter;

        let Self::QueryParams {
            order_by,
            pagination: Pagination { limit, offset },
            ..
        } = $query;

        let query = $query.as_diesel_filter();

        let mut statement = Self::as_diesel_query_base()
            .select(Self::as_select())
            .limit(*limit)
            .offset(*offset)
            .into_boxed();

        if let Some(query) = query {
            statement = statement.filter(query);
        }

        for ordering in order_by {
            statement = match (ordering.column, ordering.descending) {
                $(
                    ($ordinal_col_enum_variant, false) => statement.then_order_by($corresponding_db_col.asc()),
                    ($ordinal_col_enum_variant, true) => statement.then_order_by($corresponding_db_col.desc()),
                )*
            };
        }

        Ok(statement.load($db_conn).await?)
    }};
}
