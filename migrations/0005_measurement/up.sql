-- Your SQL goes here
-- This type appears in the primary key of many tables as (item_id, measured_by, measurement) so that the same person
-- doesn't insert the same measurement for the same item twice
create type measurement as (
    quantity text,
    value real,
    unit text, -- constrained by Rust enum
    measured_at timestamp,
    instrument_name text, -- constrained by Rust enum
    notes text []
);
