-- Your SQL goes here
create table multiplexing_tag_type (
    id uuid primary key default gen_random_uuid(),
    name text not null unique --constrained by Rust enum, but can be updated if needed
);

create table multiplexing_tag (
    id uuid primary key default gen_random_uuid(),
    tag_name text not null,
    -- constrained by Rust-side enum
    type_id uuid references multiplexing_tag_type on update restrict on delete restrict not null,
    unique (tag_name, type_id)
);

create table suspension (
    id uuid primary key default gen_random_uuid(),
    link text generated always as ('/samples/' || id) stored not null,
    legacy_id text unique not null,
    metadata_id uuid references sample_metadata on delete restrict on update restrict,
    parent_specimen_id uuid references specimen on delete restrict on update restrict,
    is_derived boolean generated always as (parent_specimen_id is not null) stored,
    biological_material text not null, -- constrained by Rust-side enum
    created_at timestamp,
    pooled_into_id uuid references multiplexed_suspension on delete restrict on update restrict,
    multiplexing_tag_id uuid references multiplexing_tag on delete restrict on update restrict,
    targeted_cell_recovery real not null, -- validated on Rust side
    target_reads_per_cell integer not null, -- validated on Rust side
    notes text [],

    -- a derived suspension must not have its own metadata
    constraint has_metadata check (is_derived = (metadata_id is null)),
    -- a derived suspension must have a creation date
    constraint has_creation_time check (is_derived = (created_at is not null)),
    -- either both are specified or neither is specified
    constraint pooling_is_correctly_specified check ((pooled_into_id is null) = (multiplexing_tag_id is null))
);

create table suspension_measurement (
    id uuid primary key default gen_random_uuid(),
    suspension_id uuid references suspension on delete restrict on update restrict not null,
    measured_by uuid references person on delete restrict on update restrict not null,
    data jsonb not null
);

create table suspension_preparers (
    suspension_id uuid references suspension on delete restrict on update restrict not null,
    prepared_by uuid references person on delete restrict on update restrict not null,

    primary key (suspension_id, prepared_by)
);
