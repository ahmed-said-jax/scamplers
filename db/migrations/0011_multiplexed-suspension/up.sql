create table multiplexed_suspension (
    id uuid primary key default uuidv7(),
    link text generated always as ('/samples/' || id) stored not null,
    name text not null,
    readable_id text unique not null,
    pooled_at timestamptz not null,
    notes text []
);

create table multiplexed_suspension_measurement (
    id uuid primary key default uuidv7(),
    suspension_id uuid references multiplexed_suspension on delete restrict on update restrict not null,
    measured_by uuid references person on delete restrict on update restrict not null,
    data jsonb not null
);

create table multiplexed_suspension_preparers (
    suspension_id uuid references multiplexed_suspension on delete restrict on update restrict not null,
    prepared_by uuid references person on delete restrict on update restrict not null,
    primary key (suspension_id, prepared_by)
);
