use std::iter::zip;

use crate::models::{DataSet, Lab};
use anyhow::{Context, Result};
use mongodb::{
    bson::{doc, to_bson, Document},
    options::FindOneAndUpdateOptions,
    sync::{Client, Collection, Database},
};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

pub fn get_db(db_uri: &String, db_name: &String) -> Result<Database> {
    let client = Client::with_uri_str(db_uri)?;
    Ok(client.database(&db_name)) //TODO: add permissions and roles and a username/password authentication here using ClientOptions and Credentials
}

fn upsert_many<T: DeserializeOwned + Serialize + Debug>(
    collection: &Collection<T>,
    data: Vec<T>,
    filters: Vec<Document>,
) -> Result<()> {
    let options = FindOneAndUpdateOptions::builder()
        .upsert(Some(true))
        .build();

    for (item, filter) in zip(data, filters) {
        let item = to_bson(&item).with_context(|| {
            format!("could not convert the following data to BSON:\n{:#?}", item)
        })?;

        let update = doc! {"$set": item.clone()};

        collection
            .find_one_and_update(filter, update, options.clone())
            .with_context(|| {
                format!("could not upsert the following data:\n{:#?}", item.clone())
            })?;
    }

    Ok(())
}

pub fn upsert_data_sets(collection: &Collection<DataSet>, data_sets: Vec<DataSet>) -> Result<()> {
    let mut filters: Vec<Document> = Vec::new();

    for data_set in &data_sets {
        let library_ids: Vec<String> = data_set
            .libraries
            .iter()
            .map(|lib| lib._id.clone())
            .collect();

        let filter = doc! {"libraries": {"$elemMatch": { "_id": { "$in": library_ids } }}};

        filters.push(filter);
    }

    upsert_many(collection, data_sets, filters)?;

    Ok(())
}

pub fn upsert_labs(collection: &Collection<Lab>, labs: Vec<Lab>) -> Result<()> {
    let mut filters: Vec<Document> = Vec::new();

    for lab in &labs {
        let filter = doc! {"name": lab.name.clone()};
        filters.push(filter);
    }

    upsert_many(collection, labs, filters)?;

    Ok(())
}

pub fn get_data_set(collection: &Collection<DataSet>) {}