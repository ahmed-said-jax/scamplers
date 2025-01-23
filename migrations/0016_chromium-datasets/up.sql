-- Your SQL goes here
create table chromium_datasets (
    id uuid primary key,
    metadata_id uuid references dataset_metadata on delete restrict on update restrict not null,
    chemistry_name text references chemistries
);

create table chromium_dataset_composition (
    dataset_id uuid references chromium_datasets on delete restrict on update restrict not null,
    library_id uuid references chromium_libraries on delete restrict on update restrict not null,
    primary key (dataset_id, library_id)
);
