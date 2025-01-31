-- Your SQL goes here
create table lab (
    id uuid primary key default gen_random_uuid(),
    link text generated always as ('/api/labs/' || id) stored not null,
    name text unique not null,
    pi_id uuid references person on delete restrict on update restrict not null,
    delivery_dir text unique not null
);

create table lab_membership (
    lab_id uuid references lab on delete restrict on update restrict not null,
    member_id uuid references person on delete restrict on update restrict not null,
    primary key (lab_id, member_id)
);
