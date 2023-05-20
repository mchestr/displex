-- All existing users are subs, so set default to true then drop
ALTER TABLE plex_users
ADD is_subscriber BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE plex_users
ALTER COLUMN is_subscriber DROP DEFAULT;
