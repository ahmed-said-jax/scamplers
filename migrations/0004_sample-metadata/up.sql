-- Your SQL goes here
create type species as enum (
    'ambystoma_mexicanum',
    'canis_familiaris',
    'drosophila_melanogaster',
    'gasterosteus_aculeatus',
    'homo_sapiens',
    'mus_musculus',
    'rattus_norvegicus',
    'sminthopsis_crassicaudata'
);


create table sample_metadata (
    id uuid primary key,
    name text not null,
    submitted_by uuid references people on delete restrict on update restrict,
    lab_id uuid references labs on delete restrict on update restrict not null,
    received_at timestamp not null,
    species species [] not null,
    tissue text not null,
    experimental_notes text,
    returned_at timestamp,
    returned_by uuid references people on delete restrict on update restrict
);

create type committee_type as enum (
    'institutional_animal_care_and_use_committee',
    'institutional_biosafety_committee',
    'institutional_review_board'
);

create table committee_approvals (
    institution_id uuid references institutions on delete restrict on update restrict not null,
    sample_id uuid references sample_metadata on delete restrict on update restrict not null,
    committee_type committee_type not null, -- rely on the appilcation to constrain this to a defined list
    compliance_identifier text not null,
    primary key (institution_id, committee_type, sample_id)
);
