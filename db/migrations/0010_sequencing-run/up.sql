create table sequencing_run (
    id uuid primary key default gen_random_uuid(),
    link text generated always as ('/sequencing_runs/' || id) stored not null,
    legacy_id text unique not null,
    begun_at timestamp not null,
    finished_at timestamp,
    notes text []
);
