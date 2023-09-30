-- Add down migration script here
ALTER TABLE discord_user DROP COLUMN is_active;
