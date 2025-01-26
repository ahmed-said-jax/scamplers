// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "measurement"))]
    pub struct Measurement;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "parsed_metrics_file"))]
    pub struct ParsedMetricsFile;
}

diesel::table! {
    cdna (id) {
        id -> Uuid,
        legacy_id -> Text,
        prepared_at -> Timestamp,
        gems_id -> Uuid,
        specification_id -> Uuid,
        storage_location -> Nullable<Text>,
        notes -> Nullable<Array<Nullable<Text>>>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Measurement;

    cdna_measurements (cdna_id, measured_by, measurement) {
        cdna_id -> Uuid,
        measured_by -> Uuid,
        measurement -> Measurement,
    }
}

diesel::table! {
    cdna_preparers (cdna_id, prepared_by) {
        cdna_id -> Uuid,
        prepared_by -> Uuid,
    }
}

diesel::table! {
    chemistries (name) {
        name -> Text,
        description -> Text,
        definition -> Jsonb,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Measurement;

    chip_loading (gem_id, suspension_id, multiplexed_suspension_id) {
        gem_id -> Uuid,
        suspension_id -> Uuid,
        multiplexed_suspension_id -> Uuid,
        suspension_volume_loaded -> Measurement,
        buffer_volume_loaded -> Measurement,
        notes -> Nullable<Array<Nullable<Text>>>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ParsedMetricsFile;

    chromium_datasets (id) {
        id -> Uuid,
        metadata_id -> Uuid,
        gems_id -> Uuid,
        metrics -> Nullable<Array<Nullable<ParsedMetricsFile>>>,
    }
}

diesel::table! {
    chromium_libraries (id) {
        id -> Uuid,
        legacy_id -> Text,
        cdna_id -> Uuid,
        single_index_set_name -> Nullable<Text>,
        dual_index_set_name -> Nullable<Text>,
        number_of_sample_index_pcr_cycles -> Int4,
        prepared_at -> Timestamp,
        notes -> Nullable<Array<Nullable<Text>>>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Measurement;

    chromium_library_measurements (library_id, measured_by, measurement) {
        library_id -> Uuid,
        measured_by -> Uuid,
        measurement -> Measurement,
    }
}

diesel::table! {
    chromium_library_preparers (library_id, prepared_by) {
        library_id -> Uuid,
        prepared_by -> Uuid,
    }
}

diesel::table! {
    chromium_runners (run_id, run_by) {
        run_id -> Uuid,
        run_by -> Uuid,
    }
}

diesel::table! {
    chromium_runs (id) {
        id -> Uuid,
        legacy_id -> Text,
        chip -> Text,
        run_at -> Timestamp,
        succeeded -> Bool,
        notes -> Nullable<Array<Nullable<Text>>>,
    }
}

diesel::table! {
    chromium_sequencing_submissions (library_id, sequencing_run_id) {
        library_id -> Uuid,
        sequencing_run_id -> Uuid,
        submitted_at -> Timestamp,
    }
}

diesel::table! {
    committee_approvals (institution_id, committee_type, sample_id) {
        institution_id -> Uuid,
        sample_id -> Uuid,
        committee_type -> Text,
        compliance_identifier -> Text,
    }
}

diesel::table! {
    dataset_metadata (id) {
        id -> Uuid,
        name -> Text,
        lab_id -> Uuid,
        data_path -> Nullable<Text>,
        delivered_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    dual_index_sets (name) {
        name -> Text,
        kit -> Text,
        well -> Text,
        index_i7 -> Text,
        index2_workflow_a_i5 -> Text,
        index2_workflow_b_i5 -> Text,
    }
}

diesel::table! {
    gems (id) {
        id -> Uuid,
        legacy_id -> Text,
        chromium_run_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    index_kits (name) {
        name -> Text,
    }
}

diesel::table! {
    institutions (id) {
        id -> Uuid,
        name -> Text,
        ms_tenant_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    lab_membership (lab_id, member_id) {
        lab_id -> Uuid,
        member_id -> Uuid,
    }
}

diesel::table! {
    labs (id) {
        id -> Uuid,
        name -> Text,
        pi_id -> Uuid,
        delivery_dir -> Text,
    }
}

diesel::table! {
    library_type_specifications (id) {
        id -> Uuid,
        library_type -> Text,
        chemistry_name -> Text,
        index_kit -> Text,
        #[sql_name = "cdna_volume__µl"]
        cdna_volume_l -> Float4,
        #[sql_name = "library_volume__µl"]
        library_volume_l -> Float4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Measurement;

    multiplexed_suspension_measurements (suspension_id, measured_by, measurement) {
        suspension_id -> Uuid,
        measured_by -> Uuid,
        measurement -> Measurement,
    }
}

diesel::table! {
    multiplexed_suspension_preparers (suspension_id, prepared_by) {
        suspension_id -> Uuid,
        prepared_by -> Uuid,
    }
}

diesel::table! {
    multiplexed_suspensions (id) {
        id -> Uuid,
        legacy_id -> Text,
        date_pooled -> Date,
        tag_type -> Text,
        notes -> Nullable<Array<Nullable<Text>>>,
    }
}

diesel::table! {
    multiplexing_tags (id) {
        id -> Uuid,
        tag_id -> Text,
        #[sql_name = "type"]
        type_ -> Text,
    }
}

diesel::table! {
    people (id) {
        id -> Uuid,
        first_name -> Text,
        last_name -> Text,
        email -> Text,
        orcid -> Nullable<Text>,
        ms_user_id -> Nullable<Uuid>,
        api_key -> Nullable<Uuid>,
        institution_id -> Uuid,
    }
}

diesel::table! {
    sample_metadata (id) {
        id -> Uuid,
        name -> Text,
        submitted_by -> Nullable<Uuid>,
        lab_id -> Uuid,
        received_at -> Timestamp,
        species -> Array<Nullable<Text>>,
        tissue -> Text,
        experimental_notes -> Nullable<Text>,
        returned_at -> Nullable<Timestamp>,
        returned_by -> Nullable<Uuid>,
    }
}

diesel::table! {
    sequencing_runs (id) {
        id -> Uuid,
        legacy_id -> Text,
        begun_at -> Timestamp,
        finished_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    single_index_sets (name) {
        name -> Text,
        kit -> Text,
        well -> Text,
        sequences -> Array<Nullable<Text>>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Measurement;

    specimen_measurements (specimen_id, measured_by, measurement) {
        specimen_id -> Uuid,
        measured_by -> Uuid,
        measurement -> Measurement,
    }
}

diesel::table! {
    specimens (id) {
        id -> Uuid,
        legacy_id -> Text,
        metadata_id -> Nullable<Uuid>,
        #[sql_name = "type"]
        type_ -> Text,
        derived_from -> Nullable<Uuid>,
        derived_at -> Nullable<Timestamp>,
        embedded_in -> Text,
        preservation_method -> Text,
        notes -> Nullable<Array<Nullable<Text>>>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Measurement;

    suspension_measurements (suspension_id, measured_by, measurement) {
        suspension_id -> Uuid,
        measured_by -> Uuid,
        measurement -> Measurement,
        post_hybridization -> Bool,
    }
}

diesel::table! {
    suspension_preparers (suspension_id, prepared_by) {
        suspension_id -> Uuid,
        prepared_by -> Uuid,
    }
}

diesel::table! {
    suspensions (id) {
        id -> Uuid,
        legacy_id -> Text,
        metadata_id -> Nullable<Uuid>,
        parent_specimen_id -> Nullable<Uuid>,
        parent_suspension_id -> Nullable<Uuid>,
        is_derived -> Nullable<Bool>,
        biological_material -> Text,
        buffer -> Text,
        created_at -> Nullable<Timestamp>,
        pooled_into_id -> Nullable<Uuid>,
        multiplexing_tag_id -> Nullable<Uuid>,
        targeted_cell_recovery -> Float4,
        target_reads_per_cell -> Int4,
        notes -> Nullable<Array<Nullable<Text>>>,
    }
}

diesel::joinable!(cdna -> gems (gems_id));
diesel::joinable!(cdna -> library_type_specifications (specification_id));
diesel::joinable!(cdna_measurements -> cdna (cdna_id));
diesel::joinable!(cdna_measurements -> people (measured_by));
diesel::joinable!(cdna_preparers -> cdna (cdna_id));
diesel::joinable!(cdna_preparers -> people (prepared_by));
diesel::joinable!(chip_loading -> gems (gem_id));
diesel::joinable!(chip_loading -> multiplexed_suspensions (multiplexed_suspension_id));
diesel::joinable!(chip_loading -> suspensions (suspension_id));
diesel::joinable!(chromium_datasets -> dataset_metadata (metadata_id));
diesel::joinable!(chromium_datasets -> gems (gems_id));
diesel::joinable!(chromium_libraries -> cdna (cdna_id));
diesel::joinable!(chromium_libraries -> dual_index_sets (dual_index_set_name));
diesel::joinable!(chromium_libraries -> single_index_sets (single_index_set_name));
diesel::joinable!(chromium_library_measurements -> chromium_libraries (library_id));
diesel::joinable!(chromium_library_measurements -> people (measured_by));
diesel::joinable!(chromium_library_preparers -> chromium_libraries (library_id));
diesel::joinable!(chromium_library_preparers -> people (prepared_by));
diesel::joinable!(chromium_runners -> chromium_runs (run_id));
diesel::joinable!(chromium_runners -> people (run_by));
diesel::joinable!(chromium_sequencing_submissions -> chromium_libraries (library_id));
diesel::joinable!(chromium_sequencing_submissions -> sequencing_runs (sequencing_run_id));
diesel::joinable!(committee_approvals -> institutions (institution_id));
diesel::joinable!(committee_approvals -> sample_metadata (sample_id));
diesel::joinable!(dataset_metadata -> labs (lab_id));
diesel::joinable!(dual_index_sets -> index_kits (kit));
diesel::joinable!(gems -> chromium_runs (chromium_run_id));
diesel::joinable!(lab_membership -> labs (lab_id));
diesel::joinable!(lab_membership -> people (member_id));
diesel::joinable!(labs -> people (pi_id));
diesel::joinable!(library_type_specifications -> chemistries (chemistry_name));
diesel::joinable!(library_type_specifications -> index_kits (index_kit));
diesel::joinable!(multiplexed_suspension_measurements -> multiplexed_suspensions (suspension_id));
diesel::joinable!(multiplexed_suspension_measurements -> people (measured_by));
diesel::joinable!(multiplexed_suspension_preparers -> multiplexed_suspensions (suspension_id));
diesel::joinable!(multiplexed_suspension_preparers -> people (prepared_by));
diesel::joinable!(people -> institutions (institution_id));
diesel::joinable!(sample_metadata -> labs (lab_id));
diesel::joinable!(single_index_sets -> index_kits (kit));
diesel::joinable!(specimen_measurements -> people (measured_by));
diesel::joinable!(specimen_measurements -> specimens (specimen_id));
diesel::joinable!(specimens -> sample_metadata (metadata_id));
diesel::joinable!(suspension_measurements -> people (measured_by));
diesel::joinable!(suspension_measurements -> suspensions (suspension_id));
diesel::joinable!(suspension_preparers -> people (prepared_by));
diesel::joinable!(suspension_preparers -> suspensions (suspension_id));
diesel::joinable!(suspensions -> multiplexed_suspensions (pooled_into_id));
diesel::joinable!(suspensions -> multiplexing_tags (multiplexing_tag_id));
diesel::joinable!(suspensions -> sample_metadata (metadata_id));
diesel::joinable!(suspensions -> specimens (parent_specimen_id));

diesel::allow_tables_to_appear_in_same_query!(
    cdna,
    cdna_measurements,
    cdna_preparers,
    chemistries,
    chip_loading,
    chromium_datasets,
    chromium_libraries,
    chromium_library_measurements,
    chromium_library_preparers,
    chromium_runners,
    chromium_runs,
    chromium_sequencing_submissions,
    committee_approvals,
    dataset_metadata,
    dual_index_sets,
    gems,
    index_kits,
    institutions,
    lab_membership,
    labs,
    library_type_specifications,
    multiplexed_suspension_measurements,
    multiplexed_suspension_preparers,
    multiplexed_suspensions,
    multiplexing_tags,
    people,
    sample_metadata,
    sequencing_runs,
    single_index_sets,
    specimen_measurements,
    specimens,
    suspension_measurements,
    suspension_preparers,
    suspensions,
);
