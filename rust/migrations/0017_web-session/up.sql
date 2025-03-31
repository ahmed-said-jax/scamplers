-- Your SQL goes here
create unlogged table cache (
    session_id_hash text primary key,
    user_id uuid references person on delete restrict on update restrict not null,
    data jsonb,
    inserted_at timestamp not null
);

create unlogged table ms_auth_flow (
    state text primary key,
    flow jsonb not null,
    expires_at timestamp not null
);
