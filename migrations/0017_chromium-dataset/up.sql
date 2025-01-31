-- Your SQL goes here
create type parsed_metrics_file as (
    filename text,
    data jsonb
);

create table chromium_dataset (
    id uuid primary key,
    metadata_id uuid references dataset_metadata on delete restrict on update restrict not null,
    gems_id uuid references gems on delete restrict on update restrict not null,
    metrics_files parsed_metrics_file [], -- validated on Rust side
    cellranger_web_summary text
);
