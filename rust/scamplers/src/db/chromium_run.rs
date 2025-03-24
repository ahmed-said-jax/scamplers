use std::{collections::HashMap, hash::RandomState};

// TODO: this file could make use of the snazzy new `JunctionStruct` trait, but it's gonna take some work to
// deconvolute and simplify things
use chrono::NaiveDateTime;
use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    dsl::count_star,
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types::{self, SqlType},
};
use diesel_async::RunQueryDsl;
use futures::FutureExt;
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    Create,
    sample::suspension_measurement::{self},
    utils::{BelongsToExt, DbEnum, JunctionStruct, Parent},
};
use crate::{
    db::utils::ParentSet,
    schema::{
        self,
        chip_loading::{self},
        chromium_run, gems, suspension,
    },
};
const N_SUSPENSIONS_PER_GEMS: usize = 4;

#[derive(
    Debug,
    Deserialize,
    Serialize,
    strum::IntoStaticStr,
    strum::EnumString,
    strum::VariantArray,
    FromSqlRow,
    AsExpression,
    SqlType,
    Default,
    Clone,
    Copy,
)]
#[diesel(sql_type = sql_types::Text)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
enum ChromiumChip {
    J,
    H,
    Q,
    GemxFx,
    #[serde(rename = "gemx_3p")]
    #[strum(serialize = "gemx_3p")]
    Gemx3p,
    #[serde(rename = "gemx_ocm_3p")]
    #[strum(serialize = "gemx_ocm_3p")]
    GemxOcm3p,
    #[serde(rename = "gemx_ocm_5p")]
    #[strum(serialize = "gemx_ocm_5p")]
    Gemx5p,
    #[default]
    Unknown,
}
impl DbEnum for ChromiumChip {}
impl FromSql<sql_types::Text, Pg> for ChromiumChip {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Self::from_sql_inner(bytes)
    }
}

impl ToSql<sql_types::Text, diesel::pg::Pg> for ChromiumChip {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
        self.to_sql_inner(out)
    }
}

#[derive(Deserialize, Insertable, Validate)]
#[diesel(table_name = schema::chromium_run, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
struct NewChromiumRun {
    legacy_id: String,
    chip: ChromiumChip,
    run_at: NaiveDateTime,
    succeeded: bool,
    notes: Option<Vec<String>>,
    run_by: Uuid,
    #[diesel(skip_insertion)]
    #[garde(dive, length(min = 1, max = 8))]
    gems: Vec<NewGems>,
}

impl Parent<NewGems> for NewChromiumRun {
    fn owned_children(&mut self) -> Vec<NewGems> {
        self.gems.drain(..).collect()
    }
}

impl Create for Vec<NewChromiumRun> {
    type Returns = ();

    async fn create(mut self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
        const N_GEMS_PER_RUN: usize = 8;
        let n_runs = self.len();

        let chromium_run_ids = diesel::insert_into(chromium_run::table)
            .values(&self)
            .returning(chromium_run::id)
            .get_results(conn)
            .await?;

        let flattened_gems = self.flatten_children_and_set_ids(&chromium_run_ids, N_GEMS_PER_RUN * n_runs);
        flattened_gems.create(conn).await?;

        Ok(())
    }
}

#[derive(Deserialize, Insertable, Validate, Clone)]
#[diesel(table_name = schema::chip_loading, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
struct ChipLoading {
    #[serde(default)]
    gems_id: Uuid,
    #[garde(dive)]
    suspension_volume_loaded: suspension_measurement::MeasurementData,
    #[garde(dive)]
    buffer_volume_loaded: suspension_measurement::MeasurementData,
    notes: Option<Vec<String>>,
}
impl BelongsToExt<NewGems> for ChipLoading {
    fn set_parent_id(&mut self, parent_id: Uuid) {
        self.gems_id = parent_id;
    }
}
impl ChipLoading {
    fn validate_volumes(&self) -> crate::db::Result<()> {
        use suspension_measurement::MeasurementData::Volume;

        use crate::db::units::VolumeUnit;

        let Self {
            suspension_volume_loaded,
            buffer_volume_loaded,
            ..
        } = self;
        let volumes = [suspension_volume_loaded, buffer_volume_loaded];

        let err = "invalid chip loading volume - quantity must be 'volume' and unit must be 'µl'".to_string();

        for v in volumes {
            let Volume { unit, .. } = v else {
                return Err(crate::db::Error::Other { message: err });
            };
            if !matches!(unit, VolumeUnit::Mcl) {
                return Err(crate::db::Error::Other { message: err });
            }
        }

        Ok(())
    }
}

#[derive(Deserialize, Insertable, Validate)]
#[diesel(table_name = schema::chip_loading, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
struct SuspensionChipLoading {
    suspension_id: Uuid,
    #[serde(flatten)]
    #[diesel(embed)]
    #[garde(dive)]
    inner: ChipLoading,
}
impl SuspensionChipLoading {
    fn validate_chip_loading(&self) -> crate::db::Result<()> {
        self.inner.validate_volumes()
    }
}
impl BelongsToExt<NewGems> for SuspensionChipLoading {
    fn set_parent_id(&mut self, parent_id: Uuid) {
        self.inner.set_parent_id(parent_id);
    }
}
impl Create for Vec<SuspensionChipLoading> {
    type Returns = ();

    async fn create(self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
        diesel::insert_into(chip_loading::table)
            .values(&self)
            .execute(conn)
            .await?;

        Ok(())
    }
}

#[derive(Deserialize, Insertable, Validate, Clone)]
#[diesel(table_name = schema::chip_loading, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
struct MultiplexedSuspensionChipLoading {
    multiplexed_suspension_id: Uuid,
    #[serde(flatten)]
    #[diesel(embed)]
    #[garde(dive)]
    inner: ChipLoading,
}
impl MultiplexedSuspensionChipLoading {
    fn validate_chip_loading(&self) -> crate::db::Result<()> {
        self.inner.validate_volumes()
    }
}
impl BelongsToExt<NewGems> for MultiplexedSuspensionChipLoading {
    fn set_parent_id(&mut self, parent_id: Uuid) {
        self.inner.set_parent_id(parent_id);
    }
}
impl Create for Vec<MultiplexedSuspensionChipLoading> {
    type Returns = ();

    async fn create(self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
        diesel::insert_into(chip_loading::table)
            .values(&self)
            .execute(conn)
            .await?;

        Ok(())
    }
}

#[derive(Deserialize, Insertable, Validate)]
#[diesel(table_name = schema::gems, check_for_backend(Pg))]
#[garde(allow_unvalidated)]
struct NewGemsCore {
    legacy_id: String,
    #[serde(skip)]
    n_samples: i32,
    chemistry: String,
    #[serde(skip)]
    chromium_run_id: Uuid,
}
impl BelongsToExt<NewChromiumRun> for NewGemsCore {
    fn set_parent_id(&mut self, parent_id: Uuid) {
        self.chromium_run_id = parent_id;
    }
}

#[derive(Deserialize, Validate)]
#[garde(allow_unvalidated)]
#[serde(tag = "plexy", rename_all = "snake_case")]
enum NewGems {
    Singleplexed {
        #[serde(flatten)]
        #[garde(dive)]
        inner: NewGemsCore,
        #[garde(dive, length(min = 1, max = N_SUSPENSIONS_PER_GEMS))]
        loading: Vec<SuspensionChipLoading>,
    },
    Multiplexed {
        #[serde(flatten)]
        #[garde(dive)]
        inner: NewGemsCore,
        #[garde(dive)]
        loading: MultiplexedSuspensionChipLoading,
    },
}
impl NewGems {
    fn validate_chip_loading(&self) -> crate::db::Result<()> {
        match self {
            Self::Singleplexed { loading, .. } => loading.iter().try_for_each(|v| v.validate_chip_loading()),
            Self::Multiplexed { loading, .. } => loading.validate_chip_loading(),
        }
    }

    fn set_n_samples(&mut self, pool_counts: &HashMap<Uuid, i64>) {
        match self {
            Self::Singleplexed { inner, loading } => {
                inner.n_samples = loading.len() as i32;
            }
            Self::Multiplexed { inner, loading } => {
                inner.n_samples = pool_counts
                    .get(&loading.multiplexed_suspension_id)
                    .copied()
                    .unwrap_or_default() as i32;
            }
        }
    }

    fn multiplexed_suspension_id(&self) -> Option<Uuid> {
        match self {
            Self::Multiplexed {
                loading:
                    MultiplexedSuspensionChipLoading {
                        multiplexed_suspension_id,
                        ..
                    },
                ..
            } => Some(*multiplexed_suspension_id),
            _ => None,
        }
    }
}

impl Parent<SuspensionChipLoading> for NewGems {
    fn owned_children(&mut self) -> Vec<SuspensionChipLoading> {
        match self {
            Self::Singleplexed {
                loading: suspensions, ..
            } => suspensions.drain(..).collect(),
            Self::Multiplexed { .. } => vec![],
        }
    }
}
impl Parent<MultiplexedSuspensionChipLoading> for NewGems {
    fn owned_children(&mut self) -> Vec<MultiplexedSuspensionChipLoading> {
        match self {
            Self::Singleplexed { .. } => vec![],
            Self::Multiplexed { loading, .. } => vec![loading.clone()], /* This clone is fine because we avoid complexity and it's a small data structure */
        }
    }
}
impl Create for Vec<NewGems> {
    type Returns = ();

    async fn create(mut self, conn: &mut diesel_async::AsyncPgConnection) -> super::Result<Self::Returns> {
        use NewGems::{Multiplexed, Singleplexed};

        self.iter().try_for_each(|g| g.validate_chip_loading())?;

        let n_gems = self.len();

        let multiplexed_suspension_ids = self.iter().filter_map(|g| g.multiplexed_suspension_id());
        let pool_counts: Vec<(Uuid, i64)> = suspension::table
            .filter(suspension::pooled_into_id.eq_any(multiplexed_suspension_ids))
            .group_by(suspension::pooled_into_id)
            .select((suspension::pooled_into_id.assume_not_null(), count_star()))
            .load(conn)
            .await?;

        let pool_counts: HashMap<_, _, RandomState> = HashMap::from_iter(pool_counts);

        for g in &mut self {
            g.set_n_samples(&pool_counts);
        }

        let inners: Vec<_> = self
            .iter()
            .map(|(Singleplexed { inner, .. } | Multiplexed { inner, .. })| inner)
            .collect();

        let gems_ids = diesel::insert_into(gems::table)
            .values(inners)
            .returning(gems::id)
            .get_results(conn)
            .await?;

        // It appears like we're doing a shit-ton of iterations here, but a given gems only returns a non-empty `Vec`
        // for one of these calls
        let suspension_chip_loads = <Self as ParentSet<_, SuspensionChipLoading>>::flatten_children_and_set_ids(
            &mut self,
            &gems_ids,
            N_SUSPENSIONS_PER_GEMS * n_gems,
        );
        suspension_chip_loads.create(conn).await?;

        let multiplexed_suspension_chip_loads =
            <Self as ParentSet<_, MultiplexedSuspensionChipLoading>>::flatten_children_and_set_ids(
                &mut self, &gems_ids, n_gems,
            );
        multiplexed_suspension_chip_loads.create(conn).await?;

        Ok(())
    }
}

impl BelongsToExt<NewChromiumRun> for NewGems {
    fn set_parent_id(&mut self, parent_id: Uuid) {
        match self {
            NewGems::Singleplexed { inner, .. } | NewGems::Multiplexed { inner, .. } => {
                inner.set_parent_id(parent_id);
            }
        }
    }
}
