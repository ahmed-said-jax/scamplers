-- This file contains index creations for a variety of tables. They are currently commented out so that we can perform
-- analysis on what actually requires indexing before we create the indexes.

-- create index sample_lab_idx on sample_metadata (lab_id);
-- create index sample_submitter_idx on sample_metadata (submitted_by);
-- create index sample_name_idx on sample_metadata (name);
-- create index sample_received_at_idx on sample_metadata (received_at);
-- create index tissue_idx on sample_metadata (tissue);
-- create index species_idx on sample_metadata (species);

-- create index comittee_approval_sample_idx on committee_approval (sample_id);

-- create index specimen_type_idx on specimen (type);
-- create index specimen_embedding_idx on specimen (embedded_in);
-- create index specimen_preservation_idx on specimen (preserved_with);
-- create index specimen_measurement_idx on specimen_measurement (specimen_id);

-- create index dataset_name_idx on dataset_metadata (name);
-- create index dataset_lab_idx on dataset_metadata (lab_id);
-- create index dataset_delivery_idx on dataset_metadata (delivered_at);

-- create index sequencing_run_start_idx on sequencing_run (begun_at);
-- create index sequencing_run_end_idx on sequencing_run (finished_at);

-- create index multiplexed_suspension_name_idx on multiplexed_suspension (name);
-- create index multiplexed_suspension_measurement_idx on multiplexed_suspension_measurement (suspension_id);
-- create index multiplexed_suspension_preparers_idx on multiplexed_suspension_preparers (suspension_id);

-- create index suspension_parent_idx on suspension (parent_specimen_id);
-- create index suspension_biological_material_idx on suspension (biological_material);
-- create index suspension_creation_idx on suspension (created_at);
-- create index suspension_pool_idx on suspension (pooled_into_id);
-- create index suspension_target_cell_recovery_idx on suspension (target_cell_recovery);
-- create index suspension_target_reads_per_cell_idx on suspension (target_reads_per_cell);

-- create index suspension_measurement_idx on suspension_measurement (suspension_id);

-- create index suspension_preparers_idx on suspension_preparers (suspension_id);

-- create index chromium_run_chip_idx on chromium_run (chip);
-- create index chromium_run_run_time_idx on chromium_run (run_at);

-- create index gems_n_samples_idx on gems (n_samples);
-- create index gems_chemistry_idx on gems (chemistry);
-- create index gems_chromium_run_idx on gems (chromium_run_id);

-- create index cdna_measurement_idx on cdna_measurement (cdna_id);

-- create index cnda_preparers_idx on cdna_preparers (cdna_id);

select 0;
