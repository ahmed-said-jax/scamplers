-- Your SQL goes here
create table single_index_sets (
    name text primary key,
    indexes_i7 text [] not null,
    constraint n_sequences check (array_length(indexes_i7, 1) = 4)
);

create table dual_index_sets (
    name text primary key,
    index_i7 text not null,
    index2_workflow_a_i5 text not null,
    index2_workflow_b_i5 text not null
);
