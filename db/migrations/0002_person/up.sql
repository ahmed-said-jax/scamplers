-- Your SQL goes here
create table person (
    id uuid primary key default gen_random_uuid(),
    link text generated always as ('/people/' || id) stored not null,
    name text not null,
    email text unique not null,
    institution_id uuid references institution on delete restrict on update restrict not null,
    orcid text unique,
    ms_user_id uuid unique,
    api_key_hash text unique
);

grant insert (name, email, institution_id, ms_user_id) on person to auth_user;
grant update (name, email, institution_id) on person to auth_user;
grant select (id) on person to auth_user;