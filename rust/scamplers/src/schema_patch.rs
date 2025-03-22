// @generated automatically by Diesel CLI.

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
        library_type -> Text,
        legacy_id -> Text,
        prepared_at -> Timestamp,
        gems_id -> Uuid,
        n_amplification_cycles -> Int4,
        storage_location -> Nullable<Text>,
        notes -> Nullable<Array<Text>>,
    }
}

diesel::table! {
    cdna_measurement (id) {
        id -> Uuid,
        cdna_id -> Uuid,
        measured_by -> Uuid,
        data -> Jsonb,
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
        library_types -> Array<Text>,
    }
}

diesel::table! {
    chip_loading (gems_id, suspension_id, multiplexed_suspension_id) {
        gems_id -> Uuid,
        suspension_id -> Uuid,
        multiplexed_suspension_id -> Uuid,
        suspension_volume_loaded -> Jsonb,
        buffer_volume_loaded -> Jsonb,
        notes -> Nullable<Array<Text>>,
    }
}

diesel::table! {
    chromium_dataset (id) {
        id -> Uuid,
        gems_id -> Uuid,
        metrics -> Nullable<Array<Jsonb>>,
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
    chromium_library_measurement (id) {
        id -> Uuid,
        library_id -> Uuid,
        measured_by -> Uuid,
        data -> Jsonb,
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
        run_by -> Uuid,
        succeeded -> Bool,
        notes -> Nullable<Array<Text>>,
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
        data_path -> Text,
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
        chemistry -> Nullable<Text>,
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
    library_type_specification (chemistry, library_type) {
        chemistry -> Text,
        library_type -> Text,
        index_kit -> Text,
        #[sql_name = "cdna_volume_µl"]
        cdna_volume_l -> Float4,
        #[sql_name = "library_volume_µl"]
        library_volume_l -> Float4,
    }
}

diesel::table! {
    multiplexed_suspension (id) {
        id -> Uuid,
        link -> Text,
        name -> Text,
        legacy_id -> Text,
        pooled_at -> Timestamp,
        notes -> Nullable<Array<Text>>,
    }
}

diesel::table! {
    multiplexed_suspension_measurement (id) {
        id -> Uuid,
        suspension_id -> Uuid,
        measured_by -> Uuid,
        data -> Jsonb,
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
        tag_id -> Text,
        #[sql_name = "type"]
        type_ -> Text,
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
        submitted_by -> Uuid,
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
        metadata_id -> Uuid,
        #[sql_name = "type"]
        type_ -> Text,
        embedded_in -> Nullable<Text>,
        preserved_with -> Nullable<Text>,
        measurements -> Nullable<Array<Jsonb>>,
        notes -> Nullable<Array<Text>>,
    }
}

diesel::table! {
    specimen_measurement (id) {
        id -> Uuid,
        specimen_id -> Uuid,
        measured_by -> Uuid,
        data -> Jsonb,
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
        created_at -> Timestamp,
        pooled_into_id -> Nullable<Uuid>,
        multiplexing_tag_id -> Nullable<Uuid>,
        lysis_duration_min -> Nullable<Float4>,
        targeted_cell_recovery -> Float4,
        target_reads_per_cell -> Int4,
        notes -> Nullable<Array<Text>>,
    }
}

diesel::table! {
    suspension_measurement (id) {
        id -> Uuid,
        suspension_id -> Uuid,
        measured_by -> Uuid,
        data -> Jsonb,
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
diesel::joinable!(cdna_measurement -> cdna (cdna_id));
diesel::joinable!(cdna_measurement -> person (measured_by));
diesel::joinable!(cdna_preparers -> cdna (cdna_id));
diesel::joinable!(cdna_preparers -> person (prepared_by));
diesel::joinable!(chip_loading -> gems (gems_id));
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
diesel::joinable!(chromium_run -> person (run_by));
diesel::joinable!(chromium_sequencing_submissions -> chromium_library (library_id));
diesel::joinable!(chromium_sequencing_submissions -> sequencing_run (sequencing_run_id));
diesel::joinable!(committee_approval -> institution (institution_id));
diesel::joinable!(committee_approval -> sample_metadata (sample_id));
diesel::joinable!(dataset_metadata -> lab (lab_id));
diesel::joinable!(dual_index_set -> index_kit (kit));
diesel::joinable!(gems -> chemistry (chemistry));
diesel::joinable!(gems -> chromium_run (chromium_run_id));
diesel::joinable!(lab -> person (pi_id));
diesel::joinable!(lab_membership -> lab (lab_id));
diesel::joinable!(lab_membership -> person (member_id));
diesel::joinable!(library_type_specification -> chemistry (chemistry));
diesel::joinable!(library_type_specification -> index_kit (index_kit));
diesel::joinable!(multiplexed_suspension_measurement -> multiplexed_suspension (suspension_id));
diesel::joinable!(multiplexed_suspension_measurement -> person (measured_by));
diesel::joinable!(multiplexed_suspension_preparers -> multiplexed_suspension (suspension_id));
diesel::joinable!(multiplexed_suspension_preparers -> person (prepared_by));
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
