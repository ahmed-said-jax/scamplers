create table chromium_library (
    id uuid primary key default uuidv7(),
    link text generated always as ('/libraries/' || id) stored not null,
    readable_id text unique not null,
    cdna_id uuid references cdna on delete restrict on update restrict not null,
    single_index_set_name text references single_index_set on delete restrict on update restrict,
    dual_index_set_name text references dual_index_set on delete restrict on update restrict,
    number_of_sample_index_pcr_cycles integer not null, -- validated on Rust side
    target_reads_per_cell integer not null,
    prepared_at timestamptz not null,
    notes text [], constraint has_index check ((single_index_set_name is null) != (dual_index_set_name is null))
);

create table chromium_library_measurement (
    id uuid primary key default uuidv7(),
    library_id uuid references chromium_library on delete restrict on update restrict not null,
    measured_by uuid references person on delete restrict on update restrict not null,
    data jsonb not null
);

create table chromium_library_preparers (
    library_id uuid references chromium_library on delete restrict on update restrict not null,
    prepared_by uuid references person on delete restrict on update restrict not null,
    primary key (library_id, prepared_by)
);

create table chromium_sequencing_submissions (
    library_id uuid references chromium_library on delete restrict on update restrict not null,
    sequencing_run_id uuid references sequencing_run on delete restrict on update restrict not null,
    fastq_paths text [],
    submitted_at timestamptz not null,
    primary key (library_id, sequencing_run_id)
);
