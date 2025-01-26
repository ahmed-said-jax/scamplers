-- Your SQL goes here
create table sample_metadata (
    id uuid primary key,
    name text not null,
    submitted_by uuid references people on delete restrict on update restrict,
    lab_id uuid references labs on delete restrict on update restrict not null,
    received_at timestamp not null,
    species text [] not null, -- constrained by Rust enum
    tissue text not null,
    experimental_notes text,
    returned_at timestamp,
    returned_by uuid references people on delete restrict on update restrict
);

create table committee_approvals (
    institution_id uuid references institutions on delete restrict on update restrict not null,
    sample_id uuid references sample_metadata on delete restrict on update restrict not null,
    committee_type text not null, -- constrained by Rust enum
    compliance_identifier text not null,
    primary key (institution_id, committee_type, sample_id)
);
