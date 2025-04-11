-- Your SQL goes here
create unlogged table session (
    hashed_id hashed_key primary key,
    csrf_token uuid default gen_random_uuid() not null,
    user_id uuid references person on delete cascade on update restrict not null
);
