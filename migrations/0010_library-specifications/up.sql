-- Your SQL goes here
create table library_type_specifications (
    id uuid primary key default gen_random_uuid(),
    library_type text not null, -- constrained by Rust enum
    chemistry_name text references chemistries not null,
    index_kit text references index_kits not null,
    cdna_volume__µl real not null, -- validated on Rust side
    library_volume__µl real not null, -- validated on Rust side
    unique (library_type, chemistry_name)
);
