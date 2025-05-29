-- Your SQL goes here
create table institution (
    id uuid primary key,
    link text generated always as ('/institutions/' || id) stored not null,
    name text unique not null
);

-- a comment to test CI again and again
