-- Your SQL goes here
create table institutions (
    id uuid primary key default gen_random_uuid(), name text unique not null, ms_tenant_id uuid
);
