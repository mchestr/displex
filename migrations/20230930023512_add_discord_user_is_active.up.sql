-- Add up migration script here
ALTER TABLE discord_users ADD COLUMN is_active BOOLEAN DEFAULT true;
