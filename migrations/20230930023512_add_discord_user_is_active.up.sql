-- Add up migration script here
ALTER TABLE discord_user ADD COLUMN is_active BOOLEAN DEFAULT true;
