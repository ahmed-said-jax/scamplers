-- Your SQL goes here
create table dataset_metadata (
    id uuid primary key,
    name text not null,
    lab_id uuid references lab on delete restrict on update restrict not null,
    data_path text, -- eventually, we can make this not null
    delivered_at timestamp
);
