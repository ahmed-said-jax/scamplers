-- This file should undo anything in `up.sql`
revoke insert on person
from
auth_user;

drop table person;
