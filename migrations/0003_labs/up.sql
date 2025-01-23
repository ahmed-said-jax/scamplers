-- Your SQL goes here
create table labs (
    id uuid primary key default gen_random_uuid(),
    name text unique not null,
    pi_id uuid references people on delete restrict on update restrict not null,
    delivery_dir text unique not null
);

create table lab_membership (
    lab_id uuid references labs on delete restrict on update restrict not null,
    member_id uuid references people on delete restrict on update restrict not null,
    primary key (lab_id, member_id)
);
