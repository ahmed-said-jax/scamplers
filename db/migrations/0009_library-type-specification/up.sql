-- comment
create table library_type_specification (
    chemistry text references chemistry on delete restrict on update restrict not null,
    library_type text not null,
    index_kit text references index_kit on delete restrict on update restrict not null,
    cdna_volume_µl real not null,
    library_volume_µl real not null,
    primary key (chemistry, library_type)
);
