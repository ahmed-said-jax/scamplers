-- Your SQL goes here
create type multiplexing_tag_type as enum (
    'probe_barcode',
    'TotalSeq-A',
    'TotalSeq-B',
    'TotalSeq-C'
);

create table multiplexed_suspensions (
    id uuid primary key,
    legacy_id text unique not null,
    date_pooled date not null,
    tag_type multiplexing_tag_type not null,
    notes text []
);

create table multiplexed_suspension_measurements (
    suspension_id uuid references multiplexed_suspensions on delete restrict on update restrict not null,
    measured_by uuid references people on delete restrict on update restrict not null,
    measured_at timestamp not null,
    measurement measurement not null,
    primary key (suspension_id, measured_by, measured_at)
);

create table multiplexed_suspension_preparers (
    suspension_id uuid references multiplexed_suspensions on delete restrict on update restrict not null,
    prepared_by uuid references people on delete restrict on update restrict not null,
    primary key (suspension_id, prepared_by)
);
