-- Your SQL goes here
create unlogged table session (
    hashed_id hashed_key primary key,
    -- csrf_token text unique not null,
    user_id uuid references person on delete cascade on update restrict not null
);

grant
select
    on session to login_user;

grant insert on session to login_user;

create unlogged table ms_auth_flow (
    state text primary key,
    flow jsonb not null,
    redirected_from text not null,
    expires_at timestamp not null
);

grant
select
    on ms_auth_flow to auth_user;

grant insert on ms_auth_flow to auth_user;

grant delete on ms_auth_flow to auth_user;
