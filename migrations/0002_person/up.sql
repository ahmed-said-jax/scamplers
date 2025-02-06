-- Your SQL goes here
create table person (
    id uuid primary key default gen_random_uuid(),
    link text generated always as ('/people/' || id) stored not null,
    first_name text not null,
    last_name text not null,
    full_name text generated always as (first_name || ' ' || last_name) stored not null,
    email text unique not null,
    institution_id uuid references institution on delete restrict on update restrict not null,
    orcid text unique,
    ms_user_id uuid unique,
    api_key_hash text unique
);
