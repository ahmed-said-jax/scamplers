pub mod error;
pub mod model;
pub mod seed_data;
mod util;

use diesel_async::AsyncPgConnection;
use util::{BoxedDieselExpression, QueryLimit};

trait AsDieselFilter<Table = ()> {
    fn as_diesel_filter<'a>(&'a self) -> Option<BoxedDieselExpression<'a, Table>>
    where
        Table: 'a;

    fn _limit(&self) -> QueryLimit {
        QueryLimit::default()
    }
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
