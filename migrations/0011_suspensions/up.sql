-- Your SQL goes here
create table multiplexing_tags (
    id uuid primary key default gen_random_uuid(),
    tag_id text not null,
    type multiplexing_tag_type not null,
    unique (id, type)
);

create table suspensions (
    id uuid primary key,
    legacy_id text unique not null,
    metadata_id uuid references sample_metadata on delete restrict on update restrict,
    parent_specimen_id uuid references specimens on delete restrict on update restrict,
    parent_suspension_id uuid references suspensions on delete restrict on update restrict,
    is_derived boolean generated always as (
        (parent_specimen_id is not null) or (parent_suspension_id is not null)
    ) stored,
    biological_material text not null, -- constrained by Rust-side enum
    buffer text not null, -- constrained by Rust-side enum
    date_created date,
    pooled_into_id uuid references multiplexed_suspensions on delete restrict on update restrict,
    multiplexing_tag_id uuid references multiplexing_tags on delete restrict on update restrict,
    targeted_cell_recovery double precision check (targeted_cell_recovery >= 0),
    notes text [],

    -- a derived suspension must not have its own metadata
    constraint has_metadata check (is_derived = (metadata_id is null)),
    -- a derived suspension must have a creation date
    constraint has_creation_date check (is_derived = (date_created is not null)),
    -- either both are specified or neither is specified
    constraint pooling_is_correctly_specified check ((pooled_into_id is null) = (multiplexing_tag_id is null))
);

create table suspension_measurements (
    suspension_id uuid references suspensions on delete restrict on update restrict not null,
    measured_by uuid references people on delete restrict on update restrict not null,
    measured_at timestamp not null,
    measurement measurement not null,
    post_hybridization boolean not null,
    cell_counter text,
    counting_method text,
    primary key (suspension_id, measured_by, measured_at)
);

create table suspension_preparers (
    suspension_id uuid references suspensions on delete restrict on update restrict not null,
    prepared_by uuid references people on delete restrict on update restrict not null,
    primary key (suspension_id, prepared_by)
);
