-- Your SQL goes here
create table multiplexed_suspension (
    id uuid primary key default gen_random_uuid(),
    link text generated always as ('/samples/' || id) stored not null,
    name text not null,
    legacy_id text unique not null,
    pooled_at timestamp not null,
    notes text []
);

create table multiplexed_suspension_measurement (
    id uuid primary key default gen_random_uuid(),
    suspension_id uuid references multiplexed_suspension on delete restrict on update restrict not null,
    measured_by uuid references person on delete restrict on update restrict not null,
    data jsonb not null
);

create index idx_multiplexed_suspension_measurement on multiplexed_suspension_measurement (suspension_id);

create table multiplexed_suspension_preparers (
    suspension_id uuid references multiplexed_suspension on delete restrict on update restrict not null,
    prepared_by uuid references person on delete restrict on update restrict not null,
    primary key (suspension_id, prepared_by)
);
