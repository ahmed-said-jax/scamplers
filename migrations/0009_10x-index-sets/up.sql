-- The string patterns of index kits, names, wells, and sequences will all be validated on Rust side
create table index_kits (
    name text primary key
);

create table single_index_sets (
    name text primary key,
    kit text references index_kits not null,
    well text not null,
    sequences text [] not null
);

create table dual_index_sets (
    name text primary key,
    kit text references index_kits not null,
    well text not null,
    index_i7 text not null,
    index2_workflow_a_i5 text not null,
    index2_workflow_b_i5 text not null
);
