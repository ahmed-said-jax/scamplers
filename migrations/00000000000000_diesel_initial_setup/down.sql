-- This file was automatically created by Diesel to setup helper functions
-- and other internal bookkeeping. This file is safe to edit, any future
-- changes will be added to existing projects as new migrations.

drop function if exists get_user_roles(user_id uuid);
drop function if exists create_user_if_not_exists(user_id uuid, roles text []);
drop function if exists revoke_roles_from_user(user_id uuid, roles text []);
drop function if exists grant_roles_to_user(user_id uuid, roles text []);
drop function if exists user_exists(user_id uuid);

drop role auth;
drop role login_user;

drop role computational_staff;
drop role biology_staff;
drop role app_admin;

drop function if exists diesel_manage_updated_at(_tbl regclass);
drop function if exists diesel_set_updated_at();
