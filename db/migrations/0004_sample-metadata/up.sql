-- Your SQL goes here
create table sample_metadata (
    id uuid primary key default gen_random_uuid(),
    name text not null,
    submitted_by uuid references person on delete restrict on update restrict not null,
    lab_id uuid references lab on delete restrict on update restrict not null,
    received_at timestamp not null,
    species text [] not null, -- constrained by Rust enum
    tissue text not null,
    notes text [],
    returned_at timestamp,
    returned_by uuid references person on delete restrict on update restrict
);

create index idx_sample_lab on sample_metadata (lab_id);

create table committee_approval (
    institution_id uuid references institution on delete restrict on update restrict not null,
    sample_id uuid references sample_metadata on delete restrict on update restrict not null,
    committee_type text not null, -- constrained by Rust enum
    compliance_identifier text not null,
    primary key (institution_id, committee_type, sample_id)
);
