-- This file should undo anything in `up.sql`
revoke insert on ms_auth_flow
from
auth_user;

revoke
select
on ms_auth_flow
from
auth_user;

drop table ms_auth_flow;

revoke insert on session
from
auth_user;

drop table session;
