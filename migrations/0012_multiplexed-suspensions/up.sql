-- Your SQL goes here
create table multiplexed_suspensions (
    id uuid primary key,
    legacy_id text unique not null,
    date_pooled date not null,
    tag_type text not null, -- constrained by Rust enum
    notes text []
);

create table multiplexed_suspension_measurements (
    suspension_id uuid references multiplexed_suspensions on delete restrict on update restrict not null,
    measured_by uuid references people on delete restrict on update restrict not null,
    measurement measurement not null,
    primary key (suspension_id, measured_by, measurement)
);

create table multiplexed_suspension_preparers (
    suspension_id uuid references multiplexed_suspensions on delete restrict on update restrict not null,
    prepared_by uuid references people on delete restrict on update restrict not null,
    primary key (suspension_id, prepared_by)
);
