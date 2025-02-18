insert into institution (id, name) values (
    '01921248-3fe4-739f-a0b7-4d9d87a4bea7',
    'Hogwarts School of Witchcraft and Wizardry'
);

insert into institution (id, name) values (
    '0192b4ec-80b5-70b0-92f0-78cdc6c75a09',
    'Xavier''s School for Gifted Youngsters'
);

insert into person (
    id, first_name, last_name, email, institution_id
) values (
    '0192124b-4895-704a-8162-4b5b9cc9408c',
    'Thomas',
    'Anderson',
    'thomas.anderson@neo.com',
    '01921248-3fe4-739f-a0b7-4d9d87a4bea7'
),
(
    '03bf73a7-9298-4651-8237-f401a6a824a2',
    'Peter',
    'Parker',
    'peter.parker@spiderman.com',
    '01921248-3fe4-739f-a0b7-4d9d87a4bea7'
);

insert into lab (id, name, pi_id, delivery_dir) values (
    '0192124b-f34a-776f-b82a-bad6e854c4e1',
    'Emmett Brown Lab',
    '0192124b-4895-704a-8162-4b5b9cc9408c',
    'back_to_the_future'
);

insert into lab_membership (lab_id, member_id) values (
    '0192124b-f34a-776f-b82a-bad6e854c4e1', '0192124b-4895-704a-8162-4b5b9cc9408c'
),
('0192124b-f34a-776f-b82a-bad6e854c4e1', '03bf73a7-9298-4651-8237-f401a6a824a2');
