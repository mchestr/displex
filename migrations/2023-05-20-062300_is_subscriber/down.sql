-- This file should undo anything in `up.sql`
ALTER TABLE plex_users
DROP COLUMN is_subscriber;