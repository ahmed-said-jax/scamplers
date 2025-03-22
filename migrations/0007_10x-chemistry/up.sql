-- Your SQL goes here
create table chemistry (
    name text primary key, -- we use name as primary key because that will not change
    description text not null,
    definition jsonb not null,
    library_types text [] not null
);
