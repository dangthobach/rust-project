-- PostgreSQL: cấp quyền DML/sequence cho crm_app trên schema public (chạy SAU khi migrate schema)
--
-- Bắt buộc kết nối vào database crm:
--   psql -U crm_admin -d crm -v ON_ERROR_STOP=1 -f 02_grant_app_after_migrations.sql
--
-- Migration (sqlx/backend) nên chạy bằng crm_admin; runtime dùng crm_app trong DATABASE_URL.

GRANT USAGE ON SCHEMA public TO crm_app;

GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO crm_app;

GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO crm_app;

-- Bảng/sequence tạo sau này bởi crm_admin (owner migration)
ALTER DEFAULT PRIVILEGES FOR ROLE crm_admin IN SCHEMA public
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO crm_app;

ALTER DEFAULT PRIVILEGES FOR ROLE crm_admin IN SCHEMA public
  GRANT USAGE, SELECT ON SEQUENCES TO crm_app;
