-- This file was automatically created by Diesel to setup helper functions
-- and other internal bookkeeping. This file is safe to edit, any future
-- changes will be added to existing projects as new migrations.

drop type hashed_key;

drop role login_user, computational_staff, biology_staff, app_admin;

drop function get_user_roles(user_id text);
drop function create_user_if_not_exists(user_id text, roles text []);
drop function revoke_roles_from_user(user_id text, roles text []);
drop function grant_roles_to_user(user_id text, roles text []);
drop function user_exists(user_id text);

drop function if exists diesel_manage_updated_at(_tbl regclass);
drop function if exists diesel_set_updated_at();
