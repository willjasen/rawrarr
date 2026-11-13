-- Administrative opt-in migration, not an application SQLx migration.
-- Run separately in each coordinator's rawrarr database as rawrarr.
\set ON_ERROR_STOP on
BEGIN;

DO $$
BEGIN
    IF current_user <> 'rawrarr' OR current_database() <> 'rawrarr' THEN
        RAISE EXCEPTION 'Run as rawrarr in the rawrarr database';
    END IF;
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'rawrarr_viewer') THEN
        CREATE ROLE rawrarr_viewer LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE
            NOINHERIT NOREPLICATION NOBYPASSRLS;
        COMMENT ON ROLE rawrarr_viewer IS 'Rawrarr read-only viewer v1';
    END IF;
    IF EXISTS (
        SELECT FROM pg_roles
        WHERE rolname = 'rawrarr_viewer' AND (
            rolsuper OR rolcreatedb OR rolcreaterole OR rolinherit
            OR rolreplication OR rolbypassrls
            OR shobj_description(oid, 'pg_authid') IS DISTINCT FROM 'Rawrarr read-only viewer v1'
        )
    ) OR EXISTS (
        SELECT FROM pg_auth_members
        WHERE member = (SELECT oid FROM pg_roles WHERE rolname = 'rawrarr_viewer')
    ) OR EXISTS (
        SELECT FROM pg_shdepend
        WHERE refclassid = 'pg_authid'::regclass
            AND refobjid = (SELECT oid FROM pg_roles WHERE rolname = 'rawrarr_viewer')
            AND deptype = 'o'
    ) THEN
        RAISE EXCEPTION 'Existing viewer role has unexpected privileges or ownership; review manually';
    END IF;
    IF EXISTS (
        SELECT FROM pg_proc
        WHERE prosecdef AND has_function_privilege('rawrarr_viewer', oid, 'EXECUTE')
    ) THEN
        RAISE EXCEPTION 'Viewer can execute SECURITY DEFINER functions; review before enabling access';
    END IF;
END
$$;

GRANT CONNECT ON DATABASE rawrarr TO rawrarr_viewer;
GRANT USAGE ON SCHEMA public TO rawrarr_viewer;
REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA public FROM rawrarr_viewer;
REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public FROM rawrarr_viewer;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO rawrarr_viewer;
ALTER DEFAULT PRIVILEGES FOR ROLE rawrarr IN SCHEMA public
    REVOKE ALL PRIVILEGES ON TABLES FROM rawrarr_viewer;
ALTER DEFAULT PRIVILEGES FOR ROLE rawrarr IN SCHEMA public
    GRANT SELECT ON TABLES TO rawrarr_viewer;
ALTER ROLE rawrarr_viewer SET default_transaction_read_only = on;
ALTER ROLE rawrarr_viewer SET statement_timeout = '30s';
COMMIT;
