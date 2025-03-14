use anyhow::Context;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rand::{
    Rng,
    distr::uniform::SampleRange,
    seq::{IndexedRandom, IteratorRandom},
};

use crate::{
    AppState2,
    db::{
        Create,
        index_sets::IndexSetFileUrl,
        institution::NewInstitution,
        lab::NewLab,
        person::NewPerson,
        sample::{
            NewSampleMetadata,
            specimen::{MeasurementData, NewSpecimen, NewSpecimenMeasurement},
        },
    },
};

// We use anyhow::Result here because we just want to know what went wrong, we
// don't care about serializing structured errors to a client
pub async fn download_and_insert_index_sets(app_state: AppState2, file_urls: &[IndexSetFileUrl]) -> anyhow::Result<()> {
    // Clone is fine here because everything in AppState is meant to be cloned
    // (cheaply clonable)
    let AppState2::Prod { http_client, .. } = app_state.clone() else {
        return Err(anyhow::Error::msg(
            "index sets should only be downloaded in production builds",
        ));
    };

    let downloads = file_urls
        .into_iter()
        .map(|url| url.clone().download(http_client.clone()));
    let index_sets = futures::future::try_join_all(downloads)
        .await
        .context("failed to download index set files")?;

    // A for-loop is fine because this is like 10 URLs max, and each of these is a
    // bulk insert
    let mut conn = app_state.db_conn().await?;
    for sets in &index_sets {
        sets.create(&mut conn)
            .await
            .context("failed to insert index sets into database")?;
    }

    Ok(())
}

// This might be modularized and split but I don't really see a point in doing so
pub async fn insert_test_data(app_state: AppState2) -> anyhow::Result<()> {
    let conn = &mut app_state.db_conn().await?;
    let rng = rand::rng();

    fn random_enum_choice<T: strum::VariantArray + Copy>(mut rng: impl Rng) -> T {
        *T::VARIANTS.choose(&mut rng).unwrap()
    }

    fn random_datetime(mut rng: impl Rng) -> NaiveDateTime {
        let year = (1999..2025).sample_single(&mut rng).unwrap();
        let date = (1..365).sample_single(&mut rng).unwrap();
        let secs = (0..86_400).sample_single(&mut rng).unwrap();

        NaiveDateTime::new(
            NaiveDate::from_yo_opt(year, date).unwrap(),
            NaiveTime::from_num_seconds_from_midnight_opt(secs, 0).unwrap(),
        )
    }

    fn random_string(strs: &[&str], mut rng: impl Rng) -> String {
        strs.choose(&mut rng).unwrap().to_string()
    }

    let institutions = [
        "Hogwarts School of Witchcraft and Wizardry",
        "Xavier's School for Gifted Youngsters",
    ];
    let institutions: Vec<_> = institutions
        .iter()
        .map(|name| NewInstitution {
            name: name.to_string(),
            ms_tenant_id: None,
        })
        .collect();
    let institutions = institutions.create(conn).await?;

    let random_institution_id = || institutions.choose(&mut rng.clone()).unwrap().id;

    let people = [
        ("Peter", "Parker", "spiderman@example.com"),
        ("Thomas", "Anderson", "neo@example.com"),
    ];
    let people: Vec<_> = people
        .iter()
        .map(|(first_name, last_name, email)| NewPerson {
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            email: email.to_string(),
            orcid: None,
            institution_id: random_institution_id(),
        })
        .collect();
    let people = people.create(conn).await?;

    let random_person_id = || people.choose(&mut rng.clone()).unwrap().stub.id;

    let labs = [
        ("Emmet Brown Lab", "back_to_the_future"),
        ("Rick Sanchez Lab", "rick_and_morty"),
    ];
    let labs: Vec<_> = labs
        .iter()
        .map(|(name, delivery_dir)| NewLab {
            name: name.to_string(),
            delivery_dir: delivery_dir.to_string(),
            pi_id: random_person_id(),
            member_ids: vec![],
        })
        .collect();
    let labs = labs.create(conn).await?;

    let random_lab_id = || labs.choose(&mut rng.clone()).unwrap().inner.id;

    let tissues = ["krabby patty", "skooma", "butterbeer", "scooby snacks"];

    let sample_metadatas = (0..1000).map(|i| NewSampleMetadata {
        name: format!("sample-{i}"),
        submitted_by: random_person_id(),
        lab_id: random_lab_id(),
        received_at: random_datetime(rng.clone()),
        species: vec![random_enum_choice(rng.clone())],
        tissue: random_string(&tissues, rng.clone()),
        committee_approvals: vec![],
        notes: None,
        returned_at: None,
        returned_by: None,
    });

    let random_specimen_measurement = || NewSpecimenMeasurement {
        measured_by: random_person_id(),
        data: MeasurementData::Rin {
            measured_at: random_datetime(rng.clone()),
            instrument_name: "RINometer".to_string(),
            value: (1.0..10.0).sample_single(&mut rng.clone()).unwrap(),
        },
    };
    let specimens: Vec<_> = sample_metadatas
        .enumerate()
        .map(|(i, metadata)| NewSpecimen::Block {
            metadata,
            legacy_id: format!("SP{i}"),
            measurements: vec![random_specimen_measurement()],
            notes: None,
            embedded_in: random_enum_choice(rng.clone()),
            preserved_with: random_enum_choice(rng.clone()),
        })
        .collect();

    specimens.create(conn).await?;

    Ok(())
}
