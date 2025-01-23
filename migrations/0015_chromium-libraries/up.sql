-- Your SQL goes here
create type chromium_library_type as enum (
    'Antibody Capture',
    'Antigen Capture',
    'Chromatin Accessibility',
    'CRISPR Guide Capture',
    'Custom',
    'Gene Expression',
    'Multiplexing Capture',
    'VDJ',
    'VDJ-B',
    'VDJ-T',
    'VDJ-T-GD'
);

create table chromium_libraries (
    id uuid primary key,
    legacy_id text unique not null,
    type chromium_library_type not null, -- constrained by Rust-side enum
    single_index_set_name text references single_index_sets on delete restrict on update restrict,
    dual_index_set_name text references dual_index_sets on delete restrict on update restrict,
    target_reads_per_cell integer not null check (target_reads_per_cell > 0),
    number_of_sample_index_pcr_cycles integer not null check (number_of_sample_index_pcr_cycles > 0),
    library_volume__µl double precision not null check (library_volume__µl > 0),
    prepared_at timestamp not null,
    notes text[],

    constraint has_index check ((single_index_set_name is null) != (dual_index_set_name is null))
);

create table chromium_library_measurements (
    library_id uuid references chromium_libraries on delete restrict on update restrict not null,
    measured_by uuid references people on delete restrict on update restrict not null,
    measured_at timestamp not null,
    measurement measurement not null,
    primary key (library_id, measured_by, measured_at)
);

create table chromium_library_preparers (
    library_id uuid references chromium_libraries on delete restrict on update restrict not null,
    prepared_by uuid references people on delete restrict on update restrict not null,
    primary key (library_id, prepared_by)
);

create table chromium_sequencing_submissions (
    library_id uuid references chromium_libraries on delete restrict on update restrict not null,
    sequencing_run_id uuid references sequencing_runs on delete restrict on update restrict not null,
    submitted_at timestamp not null,
    primary key (library_id, sequencing_run_id)
);
