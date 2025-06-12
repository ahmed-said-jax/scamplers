create table dataset_metadata (
    id uuid primary key default uuidv7(),
    link text generated always as ('/datasets/' || id) stored not null,
    name text not null,
    lab_id uuid references lab on delete restrict on update restrict not null,
    data_path text not null,
    delivered_at timestamp not null
);
