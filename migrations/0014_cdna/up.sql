-- Your SQL goes here
create table cdna (
    id uuid primary key,
    link text generated always as ('/cdna/' || id) stored not null,
    legacy_id text unique not null,
    prepared_at timestamp not null,
    gems_id uuid references gems not null,
    specification_id uuid references library_type_specification not null,
    storage_location text,
    notes text []
);

create table cdna_measurement (
    cdna_id uuid references cdna on delete restrict on update restrict not null,
    measured_by uuid references person on delete restrict on update restrict not null,
    data jsonb not null,

    primary key (cdna_id, measured_by, data)
);

create table cdna_preparers (
    cdna_id uuid references cdna on delete restrict on update restrict not null,
    prepared_by uuid references person on delete restrict on update restrict not null,
    primary key (cdna_id, prepared_by)
);
