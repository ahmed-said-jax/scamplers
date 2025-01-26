-- Your SQL goes here
create type parsed_metrics_file as (
    filename text,
    data jsonb
);

create table chromium_datasets (
    id uuid primary key,
    metadata_id uuid references dataset_metadata on delete restrict on update restrict not null,
    gems_id uuid references gems on delete restrict on update restrict not null,
    metrics parsed_metrics_file []
);
