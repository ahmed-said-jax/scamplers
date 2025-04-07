use anyhow::Context;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use diesel::{OptionalExtension, prelude::*};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use futures::FutureExt;
use garde::Validate;
use rand::{
    Rng,
    distr::uniform::SampleRange,
    seq::{IndexedRandom, IteratorRandom},
};
use serde::Deserialize;
use uuid::Uuid;
use valuable::Valuable;

use crate::{
    db::{
        Create, Read,
        index_sets::IndexSetFileUrl,
        institution::{Institution, NewInstitution},
        lab::NewLab,
        person::{NewPerson, UserRole, create_user_if_not_exists},
        sample::{
            NewSampleMetadata,
            specimen::{MeasurementData, NewSpecimen, NewSpecimenMeasurement, Specimen},
        },
        utils::DefaultNowNaiveDateTime,
    },
    schema,
};

#[derive(Insertable, Validate, Deserialize, Clone)]
#[diesel(table_name = schema::person, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
struct NewAdmin {
    #[garde(length(min = 1))]
    name: String,
    #[garde(email)]
    email: String,
    orcid: Option<String>,
    #[diesel(skip_insertion)]
    institution_name: String,
    #[serde(skip)]
    institution_id: Uuid,
}
impl Create for NewAdmin {
    type Returns = ();

    async fn create(mut self, conn: &mut AsyncPgConnection) -> crate::db::Result<Self::Returns> {
        use schema::{institution, person};

        let institution_id = institution::table
            .select(institution::id)
            .filter(institution::name.eq(&self.institution_name))
            .first(conn)
            .await?;

        self.institution_id = institution_id;

        let admin_id: Option<Uuid> = diesel::insert_into(person::table)
            .values(self)
            .returning(person::id)
            .on_conflict_do_nothing()
            .get_result(conn)
            .await
            .optional()?;

        if let Some(admin_id) = admin_id {
            diesel::select(create_user_if_not_exists(
                admin_id.to_string(),
                vec![UserRole::AppAdmin],
            ))
            .execute(conn)
            .await?;
        };

        Ok(())
    }
}

#[derive(Clone, Deserialize)]
#[serde(tag = "build", rename_all = "snake_case")]
pub enum SeedData {
    Dev,
    Prod {
        institutions: Vec<NewInstitution>,
        app_admin: NewAdmin,
        index_set_urls: Vec<IndexSetFileUrl>,
    },
}
impl SeedData {
    pub async fn insert(self, db_conn: &mut AsyncPgConnection, http_client: reqwest::Client) -> anyhow::Result<()> {
        match self {
            Self::Dev => create_random_data(db_conn)
                .await
                .context("failed to create and insert random data")?,
            Self::Prod {
                institutions,
                app_admin,
                index_set_urls,
            } => {
                let institutions_result = institutions.create(db_conn).await;
                if !matches!(
                    institutions_result,
                    Err(crate::db::Error::DuplicateRecord { .. }) | Ok(_)
                ) {
                    institutions_result?;
                }

                app_admin.create(db_conn).await?;
                download_and_insert_index_sets(db_conn, http_client, &index_set_urls).await?
            }
        }

        Ok(())
    }
}

// We use anyhow::Result here because we just want to know what went wrong, we
// don't care about serializing structured errors to a client
async fn download_and_insert_index_sets(
    db_conn: &mut AsyncPgConnection,
    http_client: reqwest::Client,
    file_urls: &[IndexSetFileUrl],
) -> anyhow::Result<()> {
    let downloads = file_urls
        .into_iter()
        .map(|url| url.clone().download(http_client.clone()));
    let index_sets = futures::future::try_join_all(downloads)
        .await
        .context("failed to download index set files")?;

    // A for-loop is fine because this is like 10 URLs max, and each of these is a
    // bulk insert
    for sets in index_sets {
        sets.create(db_conn)
            .await
            .context("failed to insert index sets into database")?;
    }

    Ok(())
}

async fn create_random_data(conn: &mut AsyncPgConnection) -> anyhow::Result<()> {
    let rng = rand::rng();

    fn random_enum_choice<T: strum::VariantArray + Copy>(mut rng: impl Rng) -> T {
        *T::VARIANTS.choose(&mut rng).unwrap()
    }

    fn random_datetime(mut rng: impl Rng) -> DefaultNowNaiveDateTime {
        let year = (1999..2025).sample_single(&mut rng).unwrap();
        let date = (1..365).sample_single(&mut rng).unwrap();
        let secs = (0..86_400).sample_single(&mut rng).unwrap();

        DefaultNowNaiveDateTime::from(NaiveDateTime::new(
            NaiveDate::from_yo_opt(year, date).unwrap(),
            NaiveTime::from_num_seconds_from_midnight_opt(secs, 0).unwrap(),
        ))
    }

    fn random_string(strs: &[&str], mut rng: impl Rng) -> String {
        strs.choose(&mut rng).unwrap().to_string()
    }

    // If we've already inserted specimens, then the test db is populated, so we can just return
    let specimens = Specimen::fetch_many(&Default::default(), conn).await?;
    if !specimens.is_empty() {
        return Ok(());
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
        ("Peter Parker", "spiderman@example.com"),
        ("Thomas Anderson", "neo@example.com"),
    ];
    let people: Vec<_> = people
        .iter()
        .map(|(name, email)| NewPerson {
            name: name.to_string(),
            email: email.to_string(),
            orcid: None,
            institution_id: random_institution_id(),
            // roles: vec![random_enum_choice(rng.clone())],
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
        specimen_id: Uuid::nil(),
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
            measurements: vec![
                random_specimen_measurement(),
                random_specimen_measurement(),
                random_specimen_measurement(),
            ],
            notes: None,
            embedded_in: random_enum_choice(rng.clone()),
            preserved_with: random_enum_choice(rng.clone()),
        })
        .collect();

    let _specimens = specimens.create(conn).await?;

    Ok(())
}
