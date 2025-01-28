-- Your SQL goes here
create table library_type_specification (
    id uuid primary key default gen_random_uuid(),
    library_type text not null, -- constrained by Rust enum
    chemistry_name text references chemistry not null,
    index_kit text references index_kit not null,
    cdna_volume measurement not null, -- validated on Rust side
    library_volume measurement not null, -- validated on Rust side
    unique (library_type, chemistry_name)
);
