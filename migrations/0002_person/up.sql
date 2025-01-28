-- Your SQL goes here
create table person (
    id uuid primary key default gen_random_uuid(),
    first_name text not null,
    last_name text not null,
    email text unique not null,
    orcid text unique,
    ms_user_id uuid unique,
    api_key uuid unique default gen_random_uuid(),
    institution_id uuid references institution on delete restrict on update restrict not null
);
