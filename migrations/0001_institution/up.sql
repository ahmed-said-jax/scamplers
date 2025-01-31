-- Your SQL goes here
create table institution (
    id uuid primary key default gen_random_uuid(), links jsonb [] unique not null default '{}', name text unique not null, ms_tenant_id uuid
);
