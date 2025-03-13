-- Your SQL goes here
create table chromium_dataset (
    -- use the metadata_id as the primary key for simplicity
    id uuid primary key references dataset_metadata on delete restrict on update restrict,
    gems_id uuid references gems on delete restrict on update restrict not null,
    metrics jsonb [], -- validated on Rust side to be the correct json
    cellranger_web_summary text -- validated on Rust side to be HTML
);
