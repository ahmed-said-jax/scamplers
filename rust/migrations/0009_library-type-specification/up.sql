-- Your SQL goes here
create table library_type_specification (
    chemistry text references chemistry on delete restrict on update restrict not null,
    library_type text not null, -- constrained by Rust enum
    index_kit text references index_kit on delete restrict on update restrict not null,
    cdna_volume_µl real not null, -- validated on Rust side
    library_volume_µl real not null, -- validated on Rust side
    primary key (chemistry, library_type)
);
