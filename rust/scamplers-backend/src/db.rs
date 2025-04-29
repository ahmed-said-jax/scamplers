pub mod error;
pub mod model;
pub mod seed_data;
mod util;

use diesel::BoxableExpression;
use diesel::pg::Pg;
use diesel::sql_types;
use diesel_async::AsyncPgConnection;
use util::QueryLimit;

type BoxedDieselExpression<'a, Table> =
    Box<dyn BoxableExpression<Table, Pg, SqlType = sql_types::Bool> + 'a>;

trait AsDieselFilter<Table = ()> {
    fn as_diesel_filter<'a>(&'a self) -> Option<BoxedDieselExpression<'a, Table>>
    where
        Table: 'a;

    fn limit(&self) -> QueryLimit {
        Default::default()
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
        self.as_ref().map(|q| q.as_diesel_filter()).flatten()
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
