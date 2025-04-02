-- This file should undo anything in `up.sql`
revoke
select
on institution
from
institution;

drop table institution;
