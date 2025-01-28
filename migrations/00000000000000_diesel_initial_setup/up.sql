-- This file was automatically created by Diesel to setup helper functions
-- and other internal bookkeeping. This file is safe to edit, any future
-- changes will be added to existing projects as new migrations.




-- Sets up a trigger for the given table to automatically set a column called
-- `updated_at` whenever the row is modified (unless `updated_at` was included
-- in the modified columns)
--
-- # Example
--
-- ```sql
-- CREATE TABLE users (id SERIAL PRIMARY KEY, updated_at TIMESTAMP NOT NULL DEFAULT NOW());
--
-- SELECT diesel_manage_updated_at('users');
-- ```
create or replace function diesel_manage_updated_at(_tbl regclass) returns void as $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$ language plpgsql;

create or replace function diesel_set_updated_at() returns trigger as $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$ language plpgsql;

create function create_role_if_not_exists(
    role_name text
) returns void language plpgsql volatile strict as $$
    declare role_exists boolean;
    begin
        select exists (select 1 from pg_roles where rolname = role_name) into role_exists;
        if not role_exists then 
            execute format('create role %I', role_name);
        end if;
    end;
$$;

select create_role_if_not_exists('scamplers');
select create_role_if_not_exists('scamplers_plot');
