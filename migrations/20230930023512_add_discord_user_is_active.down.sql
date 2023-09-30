-- Add down migration script here
ALTER TABLE discord_users DROP COLUMN is_active;
