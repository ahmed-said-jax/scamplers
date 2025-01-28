-- Your SQL goes here
create table sequencing_run (
    id uuid primary key,
    legacy_id text unique not null,
    begun_at timestamp not null,
    finished_at timestamp
);
