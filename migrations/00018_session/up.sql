-- Your SQL goes here
create unlogged table cache (
    session_id_hash text primary key,
    user_id uuid references person not null on delete restrict on update restrict,
    other_data jsonb,
    inserted_at timestamp not null
);
