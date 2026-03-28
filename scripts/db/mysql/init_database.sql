-- MySQL / MariaDB: tài khoản quản trị + user ứng dụng
--
-- Chạy với user có quyền tạo user (thường root):
--   mysql -u root -p < init_database.sql
--
-- Sửa mật khẩu CHANGE_ME_* trước khi chạy.

CREATE USER IF NOT EXISTS 'crm_admin'@'%' IDENTIFIED BY 'CHANGE_ME_CRM_ADMIN';
-- Toàn quyền server (giống DBA). Thu hẹp trên production: chỉ GRANT trên database crm.
GRANT ALL PRIVILEGES ON *.* TO 'crm_admin'@'%' WITH GRANT OPTION;

CREATE USER IF NOT EXISTS 'crm_app'@'%' IDENTIFIED BY 'CHANGE_ME_CRM_APP';

CREATE DATABASE IF NOT EXISTS crm CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

GRANT ALL PRIVILEGES ON crm.* TO 'crm_app'@'%';

FLUSH PRIVILEGES;
