create table sequencing_run (
    id uuid primary key default uuidv7(),
    link text generated always as ('/sequencing_runs/' || id) stored not null,
    readable_id text unique not null,
    begun_at timestamptz not null,
    finished_at timestamptz not null,
    notes text []
);
