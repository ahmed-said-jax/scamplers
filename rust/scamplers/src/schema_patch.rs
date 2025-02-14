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
    cache (session_id_hash) {
        session_id_hash -> Text,
        user_id -> Uuid,
        data -> Nullable<Jsonb>,
        inserted_at -> Timestamp,
    }
}

diesel::table! {
    cdna (id) {
        id -> Uuid,
        link -> Text,
        legacy_id -> Text,
        prepared_at -> Timestamp,
        gems_id -> Uuid,
        specification_id -> Uuid,
        storage_location -> Nullable<Text>,
        notes -> Nullable<Array<Text>>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Measurement;

    cdna_measurement (cdna_id, measured_by, measurement) {
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
    chemistry (name) {
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
        notes -> Nullable<Array<Text>>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ParsedMetricsFile;

    chromium_dataset (id) {
        id -> Uuid,
        gems_id -> Uuid,
        metrics_files -> Nullable<Array<ParsedMetricsFile>>,
        cellranger_web_summary -> Nullable<Text>,
    }
}

diesel::table! {
    chromium_library (id) {
        id -> Uuid,
        link -> Text,
        legacy_id -> Text,
        cdna_id -> Uuid,
        single_index_set_name -> Nullable<Text>,
        dual_index_set_name -> Nullable<Text>,
        number_of_sample_index_pcr_cycles -> Int4,
        prepared_at -> Timestamp,
        notes -> Nullable<Array<Text>>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Measurement;

    chromium_library_measurement (library_id, measured_by, measurement) {
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
    chromium_run (id) {
        id -> Uuid,
        link -> Text,
        legacy_id -> Text,
        chip -> Text,
        run_at -> Timestamp,
        succeeded -> Bool,
        notes -> Nullable<Array<Text>>,
    }
}

diesel::table! {
    chromium_runners (run_id, run_by) {
        run_id -> Uuid,
        run_by -> Uuid,
    }
}

diesel::table! {
    chromium_sequencing_submissions (library_id, sequencing_run_id) {
        library_id -> Uuid,
        sequencing_run_id -> Uuid,
        fastq_path -> Nullable<Text>,
        submitted_at -> Timestamp,
    }
}

diesel::table! {
    committee_approval (institution_id, committee_type, sample_id) {
        institution_id -> Uuid,
        sample_id -> Uuid,
        committee_type -> Text,
        compliance_identifier -> Text,
    }
}

diesel::table! {
    dataset_metadata (id) {
        id -> Uuid,
        link -> Text,
        name -> Text,
        lab_id -> Uuid,
        data_path -> Nullable<Text>,
        delivered_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    dual_index_set (name) {
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
        link -> Text,
        legacy_id -> Text,
        chromium_run_id -> Uuid,
    }
}

diesel::table! {
    index_kit (name) {
        name -> Text,
    }
}

diesel::table! {
    institution (id) {
        id -> Uuid,
        link -> Text,
        name -> Text,
        ms_tenant_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    lab (id) {
        id -> Uuid,
        link -> Text,
        name -> Text,
        pi_id -> Uuid,
        delivery_dir -> Text,
    }
}

diesel::table! {
    lab_membership (lab_id, member_id) {
        lab_id -> Uuid,
        member_id -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Measurement;

    library_type_specification (id) {
        id -> Uuid,
        library_type -> Text,
        chemistry_name -> Text,
        index_kit -> Text,
        cdna_volume -> Measurement,
        library_volume -> Measurement,
    }
}

diesel::table! {
    multiplexed_suspension (id) {
        id -> Uuid,
        link -> Text,
        legacy_id -> Text,
        date_pooled -> Date,
        tag_type -> Text,
        notes -> Nullable<Array<Text>>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Measurement;

    multiplexed_suspension_measurement (suspension_id, measured_by, measurement) {
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
    multiplexing_tag (id) {
        id -> Uuid,
        tag_name -> Text,
        type_id -> Uuid,
    }
}

diesel::table! {
    multiplexing_tag_type (id) {
        id -> Uuid,
        name -> Text,
    }
}

diesel::table! {
    person (id) {
        id -> Uuid,
        link -> Text,
        first_name -> Text,
        last_name -> Text,
        full_name -> Text,
        email -> Text,
        institution_id -> Uuid,
        orcid -> Nullable<Text>,
        ms_user_id -> Nullable<Uuid>,
        api_key_hash -> Nullable<Text>,
    }
}

diesel::table! {
    sample_metadata (id) {
        id -> Uuid,
        name -> Text,
        submitted_by -> Nullable<Uuid>,
        lab_id -> Uuid,
        received_at -> Timestamp,
        species -> Array<Text>,
        tissue -> Text,
        notes -> Nullable<Array<Text>>,
        returned_at -> Nullable<Timestamp>,
        returned_by -> Nullable<Uuid>,
    }
}

diesel::table! {
    sequencing_run (id) {
        id -> Uuid,
        link -> Text,
        legacy_id -> Text,
        begun_at -> Timestamp,
        finished_at -> Nullable<Timestamp>,
        notes -> Nullable<Array<Text>>,
    }
}

diesel::table! {
    single_index_set (name) {
        name -> Text,
        kit -> Text,
        well -> Text,
        sequences -> Array<Text>,
    }
}

diesel::table! {
    specimen (id) {
        id -> Uuid,
        link -> Text,
        legacy_id -> Text,
        metadata_id -> Nullable<Uuid>,
        #[sql_name = "type"]
        type_ -> Text,
        derived_from -> Nullable<Uuid>,
        derived_at -> Nullable<Timestamp>,
        embedded_in -> Text,
        preservation_method -> Text,
        notes -> Nullable<Array<Text>>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Measurement;

    specimen_measurement (specimen_id, measured_by, measurement) {
        specimen_id -> Uuid,
        measured_by -> Uuid,
        measurement -> Measurement,
    }
}

diesel::table! {
    suspension (id) {
        id -> Uuid,
        link -> Text,
        legacy_id -> Text,
        metadata_id -> Nullable<Uuid>,
        parent_specimen_id -> Nullable<Uuid>,
        is_derived -> Nullable<Bool>,
        biological_material -> Text,
        created_at -> Nullable<Timestamp>,
        pooled_into_id -> Nullable<Uuid>,
        multiplexing_tag_id -> Nullable<Uuid>,
        targeted_cell_recovery -> Float4,
        target_reads_per_cell -> Int4,
        notes -> Nullable<Array<Text>>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Measurement;

    suspension_measurement (suspension_id, measured_by, measurement) {
        suspension_id -> Uuid,
        measured_by -> Uuid,
        measurement -> Measurement,
        post_hybridization -> Bool,
        buffer -> Text,
    }
}

diesel::table! {
    suspension_preparers (suspension_id, prepared_by) {
        suspension_id -> Uuid,
        prepared_by -> Uuid,
    }
}

diesel::joinable!(cache -> person (user_id));
diesel::joinable!(cdna -> gems (gems_id));
diesel::joinable!(cdna -> library_type_specification (specification_id));
diesel::joinable!(cdna_measurement -> cdna (cdna_id));
diesel::joinable!(cdna_measurement -> person (measured_by));
diesel::joinable!(cdna_preparers -> cdna (cdna_id));
diesel::joinable!(cdna_preparers -> person (prepared_by));
diesel::joinable!(chip_loading -> gems (gem_id));
diesel::joinable!(chip_loading -> multiplexed_suspension (multiplexed_suspension_id));
diesel::joinable!(chip_loading -> suspension (suspension_id));
diesel::joinable!(chromium_dataset -> dataset_metadata (id));
diesel::joinable!(chromium_dataset -> gems (gems_id));
diesel::joinable!(chromium_library -> cdna (cdna_id));
diesel::joinable!(chromium_library -> dual_index_set (dual_index_set_name));
diesel::joinable!(chromium_library -> single_index_set (single_index_set_name));
diesel::joinable!(chromium_library_measurement -> chromium_library (library_id));
diesel::joinable!(chromium_library_measurement -> person (measured_by));
diesel::joinable!(chromium_library_preparers -> chromium_library (library_id));
diesel::joinable!(chromium_library_preparers -> person (prepared_by));
diesel::joinable!(chromium_runners -> chromium_run (run_id));
diesel::joinable!(chromium_runners -> person (run_by));
diesel::joinable!(chromium_sequencing_submissions -> chromium_library (library_id));
diesel::joinable!(chromium_sequencing_submissions -> sequencing_run (sequencing_run_id));
diesel::joinable!(committee_approval -> institution (institution_id));
diesel::joinable!(committee_approval -> sample_metadata (sample_id));
diesel::joinable!(dataset_metadata -> lab (lab_id));
diesel::joinable!(dual_index_set -> index_kit (kit));
diesel::joinable!(gems -> chromium_run (chromium_run_id));
diesel::joinable!(lab -> person (pi_id));
diesel::joinable!(lab_membership -> lab (lab_id));
diesel::joinable!(lab_membership -> person (member_id));
diesel::joinable!(library_type_specification -> chemistry (chemistry_name));
diesel::joinable!(library_type_specification -> index_kit (index_kit));
diesel::joinable!(multiplexed_suspension_measurement -> multiplexed_suspension (suspension_id));
diesel::joinable!(multiplexed_suspension_measurement -> person (measured_by));
diesel::joinable!(multiplexed_suspension_preparers -> multiplexed_suspension (suspension_id));
diesel::joinable!(multiplexed_suspension_preparers -> person (prepared_by));
diesel::joinable!(multiplexing_tag -> multiplexing_tag_type (type_id));
diesel::joinable!(person -> institution (institution_id));
diesel::joinable!(sample_metadata -> lab (lab_id));
diesel::joinable!(single_index_set -> index_kit (kit));
diesel::joinable!(specimen -> sample_metadata (metadata_id));
diesel::joinable!(specimen_measurement -> person (measured_by));
diesel::joinable!(specimen_measurement -> specimen (specimen_id));
diesel::joinable!(suspension -> multiplexed_suspension (pooled_into_id));
diesel::joinable!(suspension -> multiplexing_tag (multiplexing_tag_id));
diesel::joinable!(suspension -> sample_metadata (metadata_id));
diesel::joinable!(suspension -> specimen (parent_specimen_id));
diesel::joinable!(suspension_measurement -> person (measured_by));
diesel::joinable!(suspension_measurement -> suspension (suspension_id));
diesel::joinable!(suspension_preparers -> person (prepared_by));
diesel::joinable!(suspension_preparers -> suspension (suspension_id));

diesel::allow_tables_to_appear_in_same_query!(
    cache,
    cdna,
    cdna_measurement,
    cdna_preparers,
    chemistry,
    chip_loading,
    chromium_dataset,
    chromium_library,
    chromium_library_measurement,
    chromium_library_preparers,
    chromium_run,
    chromium_runners,
    chromium_sequencing_submissions,
    committee_approval,
    dataset_metadata,
    dual_index_set,
    gems,
    index_kit,
    institution,
    lab,
    lab_membership,
    library_type_specification,
    multiplexed_suspension,
    multiplexed_suspension_measurement,
    multiplexed_suspension_preparers,
    multiplexing_tag,
    multiplexing_tag_type,
    person,
    sample_metadata,
    sequencing_run,
    single_index_set,
    specimen,
    specimen_measurement,
    suspension,
    suspension_measurement,
    suspension_preparers,
);
