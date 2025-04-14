-- Your SQL goes here
create unlogged table session (
    hashed_id hashed_key primary key,
    user_id uuid references person on delete cascade on update restrict not null
);
