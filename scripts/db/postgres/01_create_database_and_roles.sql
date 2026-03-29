-- PostgreSQL: tạo database + 2 role đăng nhập (chạy một lần với superuser, thường là postgres)
--
--   crm_admin — quản trị cluster/DB (SUPERUSER + CREATEDB + CREATEROLE). Production: cân nhắc
--             bỏ SUPERUSER, chỉ cấp quyền trên database crm.
--   crm_app   — user ứng dụng (runtime, không superuser).
--
-- Chạy (lỗi sẽ dừng nếu dùng: psql -v ON_ERROR_STOP=1):
--   psql -U postgres -v ON_ERROR_STOP=1 -f 01_create_database_and_roles.sql
--
-- Sửa mật khẩu CHANGE_ME_* trước khi chạy.
--
-- Lưu ý: CREATE DATABASE không được bọc trong transaction; psql gửi từng câu lệnh riêng.

CREATE ROLE crm_admin WITH LOGIN PASSWORD 'CHANGE_ME_CRM_ADMIN' SUPERUSER CREATEDB CREATEROLE;

CREATE ROLE crm_app WITH LOGIN PASSWORD 'CHANGE_ME_CRM_APP' NOSUPERUSER NOCREATEDB NOCREATEROLE;

-- TEMPLATE template0: tránh lỗi encoding khi khác template1
CREATE DATABASE crm OWNER crm_admin ENCODING 'UTF8' TEMPLATE template0;

GRANT CONNECT ON DATABASE crm TO crm_app;

-- PostgreSQL 15+: schema public no longer grants CREATE to everyone. sqlx migrate with
-- DATABASE_URL=...crm_app... needs USAGE + CREATE on public (or run migrate as crm_admin).
\c crm
GRANT USAGE, CREATE ON SCHEMA public TO crm_app;
