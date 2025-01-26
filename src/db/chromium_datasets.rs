use diesel::{
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    serialize::{ToSql, WriteTuple},
    sql_types::{Jsonb, Text},
};
use uuid::Uuid;

use crate::schema::chromium_datasets;

#[derive(Debug, AsExpression)]
#[diesel(sql_type = crate::schema::sql_types::ParsedMetricsFile)]
struct ParsedMetricsFile {
    filename: String,
    data: serde_json::Value,
}
impl ToSql<crate::schema::sql_types::ParsedMetricsFile, Pg> for ParsedMetricsFile {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        WriteTuple::<(Text, Jsonb)>::write_tuple(&(&self.filename, &self.data), &mut out.reborrow())
    }
}

#[derive(Insertable)]
#[diesel(table_name = chromium_datasets, check_for_backend(Pg))]
struct NewChromiumDataset {
    id: Uuid,
    metadata_id: Uuid,
    gems_id: Uuid,
    chemistry_name: String,
    metrics: Vec<ParsedMetricsFile>,
}
impl NewChromiumDataset {
    fn create(&self, conn: &mut PgConnection) {
        diesel::insert_into(chromium_datasets::table)
            .values(self)
            .execute(conn)
            .unwrap();
    }
}
