pub mod model;
pub mod error;
pub mod seed_data;
mod util;

use diesel::prelude::*;
use diesel::BoxableExpression;
use diesel::pg::Pg;
use diesel::sql_types;
use diesel_async::AsyncPgConnection;

type BoxedDieselExpression<'a, Table> = Box<dyn BoxableExpression<Table, Pg, SqlType = sql_types::Bool> + 'a>;

trait AsDieselExpression<Table = ()> {
    fn as_diesel_expression<'a>(&'a self) -> Option<BoxedDieselExpression<'a, Table>> where Table: 'a;
}

impl AsDieselExpression for () {
    fn as_diesel_expression<'a>(&'a self) -> Option<BoxedDieselExpression<'a, ()>> where (): 'a {
        None
    }
}

impl <DbQuery, Table> AsDieselExpression<Table> for Option<DbQuery> where DbQuery: AsDieselExpression<Table> {
    fn as_diesel_expression<'a>(&'a self) -> Option<BoxedDieselExpression<'a, Table>> where Table: 'a {
        let Some(filter) = self else {
            return None;
        };

        filter.as_diesel_expression()
    }
}

trait AsDieselSelect<Table: QueryDsl> {
    fn as_diesel_select() -> Table;
}

trait Write {
    type Returns;

    async fn write(self, db_conn: &mut AsyncPgConnection) -> error::Result<Self::Returns>;
}
