-- Your SQL goes here
create table cdna (
    id uuid primary key,
    legacy_id text unique not null,
    prepared_at timestamp not null,
    volume__µl double precision not null check (volume__µl > 0),
    number_of_amplification_cycles integer not null check (number_of_amplification_cycles > 0),
    concentration__pg_per_µl double precision not null check (concentration__pg_per_µl > 0),
    total_yield__ng double precision generated always as ((volume__µl * concentration__pg_per_µl) / 1000) stored,
    storage_location text,
    notes text []
);

create table cdna_preparers (
    cdna_id uuid references cdna on delete restrict on update restrict not null,
    prepared_by uuid references people on delete restrict on update restrict not null,
    primary key (cdna_id, prepared_by)
);
