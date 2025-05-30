-- Your SQL goes here
create table person (
    id uuid primary key default gen_random_uuid (),
    link text generated always as ('/people/' || id) stored not null,
    name text not null,
    email text unique not null,
    institution_id uuid references institution on delete restrict on update restrict not null,
    orcid text unique,
    ms_user_id uuid unique,
    hashed_api_key hashed_key unique
);
