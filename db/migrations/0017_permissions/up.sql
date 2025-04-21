-- Your SQL goes here
grant select on all tables in schema public to public;
grant all on all tables in schema public to app_admin;

grant all on session to login_user;
grant insert, select, update on person to login_user;

grant insert, select, delete on ms_auth_flow to auth_user;
