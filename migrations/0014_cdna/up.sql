-- Your SQL goes here
create table cdna (
    id uuid primary key default gen_random_uuid(),
    link text generated always as ('/cdna/' || id) stored not null,
    library_type text not null, -- validated by Rust enum
    legacy_id text unique not null,
    prepared_at timestamp not null,
    gems_id uuid references gems not null,
    storage_location text,
    notes text []
);

create table cdna_measurement (
    id uuid primary key default gen_random_uuid(),
    cdna_id uuid references cdna on delete restrict on update restrict not null,
    measured_by uuid references person on delete restrict on update restrict not null,
    data jsonb not null
);

create table cdna_preparers (
    cdna_id uuid references cdna on delete restrict on update restrict not null,
    prepared_by uuid references person on delete restrict on update restrict not null,
    primary key (cdna_id, prepared_by)
);
