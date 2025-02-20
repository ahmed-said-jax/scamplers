-- Your SQL goes here
create table sequencing_run (
    id uuid primary key,
    link text generated always as ('/sequencing_runs/' || id) stored not null,
    legacy_id text unique not null,
    begun_at timestamp not null,
    finished_at timestamp,
    notes text []
);
