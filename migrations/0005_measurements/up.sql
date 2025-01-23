-- Your SQL goes here
-- units are not constrained because we will rely on Rust to ensure correctness
create type measurement as (
    quantity text,
    value double precision,
    unit text,
    notes text []
);
