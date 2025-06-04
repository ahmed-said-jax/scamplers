pub mod error;
pub mod model;
pub mod seed_data;
mod util;

use diesel_async::AsyncPgConnection;
use util::BoxedDieselExpression;

trait AsDieselFilter<Table = ()> {
    fn as_diesel_filter<'a>(&'a self) -> Option<BoxedDieselExpression<'a, Table>>
    where
        Table: 'a;
}

impl AsDieselFilter for () {
    fn as_diesel_filter<'a>(&'a self) -> Option<BoxedDieselExpression<'a, ()>>
    where
        (): 'a,
    {
        None
    }
}

impl<FilterStruct, Table> AsDieselFilter<Table> for Option<FilterStruct>
where
    FilterStruct: AsDieselFilter<Table>,
{
    fn as_diesel_filter<'a>(&'a self) -> Option<BoxedDieselExpression<'a, Table>>
    where
        Table: 'a,
    {
        self.as_ref().and_then(AsDieselFilter::as_diesel_filter)
    }
}

trait AsDieselQueryBase {
    type QueryBase;

    fn as_diesel_query_base() -> Self::QueryBase;
}

pub trait Write {
    type Returns;

    async fn write(self, db_conn: &mut AsyncPgConnection) -> error::Result<Self::Returns>;
}

pub trait FetchById: Sized {
    type Id;

    fn fetch_by_id(
        id: Self::Id,
        db_conn: &mut AsyncPgConnection,
    ) -> impl Future<Output = error::Result<Self>>;
}

pub trait FetchByFilter: Sized {
    type QueryParams;

    fn fetch_by_filter(
        query: Self::QueryParams,
        db_conn: &mut AsyncPgConnection,
    ) -> impl Future<Output = error::Result<Vec<Self>>>;
}
