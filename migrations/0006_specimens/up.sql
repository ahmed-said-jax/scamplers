-- Your SQL goes here
create type specimen_type as enum ('block', 'curl', 'tissue', 'fluid');

create type embedding_matrix as enum ('CMC', 'OCT', 'paraffin');

create type preservation_method as enum (
    'cryopreserved',
    'DSP_fixed',
    'formaldehyde_derivative_fixed',
    'frozen'
);

-- we will rely on the application to ensure that `type`, `embedding_matrix`, and `preservation_method` make sense together
create table specimens (
    id uuid primary key,
    legacy_id text unique not null,
    metadata_id uuid references sample_metadata on delete restrict on update restrict,
    type specimen_type not null,
    derived_from uuid references specimens on delete restrict on update restrict,
    derived_at timestamp,
    embedded_in embedding_matrix not null,
    preservation_method preservation_method not null,
    notes text [],

    constraint has_metadata check ((metadata_id is null) != (derived_from is null)),
    constraint derivation_fully_specified check ((derived_from is null) = (derived_at is null))
);

create table specimen_measurements (
    specimen_id uuid references specimens on delete restrict on update restrict not null,
    measured_by uuid references people on delete restrict on update restrict not null,
    measured_at timestamp not null,
    measurement measurement not null,
    primary key (specimen_id, measured_by, measured_at)
);
