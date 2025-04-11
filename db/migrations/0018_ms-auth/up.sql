create unlogged table ms_auth_flow (
    state text primary key,
    flow jsonb not null,
    redirected_from text not null,
    expires_at timestamp not null
);
