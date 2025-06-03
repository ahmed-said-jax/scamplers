-- We allow for text primary keys here because the names of these things will not change
create table index_kit (
    name text primary key
);

create table single_index_set (
    name text primary key,
    kit text references index_kit on delete restrict on update restrict not null,
    well text not null,
    sequences text [] not null
);

create table dual_index_set (
    name text primary key,
    kit text references index_kit on delete restrict on update restrict not null,
    well text not null,
    index_i7 text not null,
    index2_workflow_a_i5 text not null,
    index2_workflow_b_i5 text not null
);
