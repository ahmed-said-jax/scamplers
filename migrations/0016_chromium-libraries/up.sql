-- Your SQL goes here
create table chromium_libraries (
    id uuid primary key,
    legacy_id text unique not null,
    cdna_id uuid references cdna on delete restrict on update restrict not null,
    single_index_set_name text references single_index_sets on delete restrict on update restrict,
    dual_index_set_name text references dual_index_sets on delete restrict on update restrict,
    number_of_sample_index_pcr_cycles integer not null, -- validated on Rust side
    prepared_at timestamp not null,
    notes text [],

    constraint has_index check ((single_index_set_name is null) != (dual_index_set_name is null))
);

create table chromium_library_measurements (
    library_id uuid references chromium_libraries on delete restrict on update restrict not null,
    measured_by uuid references people on delete restrict on update restrict not null,
    measurement measurement not null,
    primary key (library_id, measured_by, measurement)
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
