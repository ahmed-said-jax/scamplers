-- Your SQL goes here
create type user_role as enum (
    'admin',
    'computational_staff',
    'lab_staff'
);

create table person (
    id uuid primary key default gen_random_uuid(),
    link text generated always as ('/api/people/' || id) stored not null,
    first_name text not null,
    last_name text not null,
    email text unique not null,
    institution_id uuid references institution on delete restrict on update restrict not null,
    roles user_role [] not null default '{}',
    orcid text unique,
    ms_user_id uuid unique,
    api_key_hash jsonb unique
);
