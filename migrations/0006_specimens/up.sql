-- Your SQL goes here
-- `type`, `embedding_matrix`, and `preservation_method` are constrained by Rust enums and will be validated to make sense together
create table specimens (
    id uuid primary key,
    legacy_id text unique not null,
    metadata_id uuid references sample_metadata on delete restrict on update restrict,
    type text not null,
    derived_from uuid references specimens on delete restrict on update restrict,
    derived_at timestamp,
    embedded_in text not null,
    preservation_method text not null,
    notes text [],

    constraint has_metadata check ((metadata_id is null) != (derived_from is null)),
    constraint derivation_fully_specified check ((derived_from is null) = (derived_at is null))
);

create table specimen_measurements (
    specimen_id uuid references specimens on delete restrict on update restrict not null,
    measured_by uuid references people on delete restrict on update restrict not null,
    measurement measurement not null,
    primary key (specimen_id, measured_by, measurement)
);
