-- Your SQL goes here
-- `type`, `embedding_matrix`, and `preservation_method` are constrained by Rust enums and will be validated to make
-- sense together
create table specimen (
    id uuid primary key default gen_random_uuid(),
    link text generated always as ('/samples/' || id) stored not null,
    legacy_id text unique not null,
    metadata_id uuid not null references sample_metadata on delete restrict on update restrict,
    type text not null,
    embedded_in text,
    preserved_with text,
    measurements jsonb[],
    notes text []
);

create table specimen_measurement (
    id uuid primary key default gen_random_uuid(),
    specimen_id uuid not null references specimen on delete restrict on update restrict,
    measured_by uuid not null references person on delete restrict on update restrict,
    data jsonb not null
);