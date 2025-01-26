-- Your SQL goes here
create table chemistries (
    name text primary key,
    description text not null,
    definition jsonb not null
);
