create table specimen (
    id uuid primary key default uuidv7(),
    link text generated always as ('/samples/' || id) stored not null,
    metadata_id uuid not null references sample_metadata on delete restrict on update restrict,
    type text not null,
    embedded_in text,
    fixative text,
    frozen bool not null default false,
    cryopreserved bool not null default false,
    storage_buffer text,

    constraint not_both_frozen_and_cryopreserved check (not (cryopreserved and frozen))
);

create table specimen_measurement (
    id uuid primary key default uuidv7(),
    specimen_id uuid not null references specimen on delete restrict on update restrict,
    measured_by uuid not null references person on delete restrict on update restrict,
    data jsonb not null
);
