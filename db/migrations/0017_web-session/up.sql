-- Your SQL goes here
create unlogged table session (
    id_hash text primary key,
    user_id uuid references person on delete restrict on update restrict not null,
    data jsonb
);

grant insert (id_hash, user_id) on session to auth_user;

create unlogged table ms_auth_flow (
    state text primary key,
    flow jsonb not null,
    redirected_from text not null,
    expires_at timestamp not null
);

grant select (state, flow, redirected_from) on ms_auth_flow to auth_user;

grant insert on ms_auth_flow to auth_user;
