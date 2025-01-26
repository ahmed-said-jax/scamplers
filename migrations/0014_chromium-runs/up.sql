-- Your SQL goes here
create table chromium_runs (
    id uuid primary key,
    legacy_id text unique not null,
    chip text not null, -- constrained by Rust enum
    run_at timestamp not null,
    succeeded boolean not null,
    notes text []
);

create table chromium_runners (
    run_id uuid references chromium_runs on delete restrict on update restrict not null,
    run_by uuid references people on delete restrict on update restrict not null,
    primary key (run_id, run_by)
);

create table gems (
    id uuid primary key,
    legacy_id text unique not null,
    chromium_run_id uuid references chromium_runs on delete restrict on update restrict
);

create table chip_loading (
    gem_id uuid references gems on delete restrict on update restrict not null,
    suspension_id uuid references suspensions on delete restrict on update restrict,
    multiplexed_suspension_id uuid references multiplexed_suspensions on delete restrict on update restrict,
    suspension_volume_loaded measurement not null, -- validated on Rust side
    buffer_volume_loaded measurement not null, -- validated on Rust side
    notes text [],
    primary key (gem_id, suspension_id, multiplexed_suspension_id),

    constraint has_suspension check ((suspension_id is null) != (multiplexed_suspension_id is null))
);
