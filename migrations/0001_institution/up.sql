-- Your SQL goes here
create table institution (
    id uuid primary key default gen_random_uuid(),
    link text generated always as ('/institutions/' || id) stored not null,
    name text unique not null, ms_tenant_id uuid
);
