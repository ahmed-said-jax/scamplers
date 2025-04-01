-- Your SQL goes here
create table institution (
    id uuid primary key default gen_random_uuid (),
    link text generated always as ('/institutions/' || id) stored not null,
    name text unique not null,
    ms_tenant_id uuid
);

insert into
    institution (id, name)
values
    (
        '00000000-0000-0000-0000-000000000000',
        '_PLACEHOLDER'
    );
