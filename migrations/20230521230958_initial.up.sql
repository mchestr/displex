CREATE OR REPLACE FUNCTION trigger_set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE IF NOT EXISTS discord_users (
  id VARCHAR PRIMARY KEY,
  username VARCHAR NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
DROP TRIGGER IF EXISTS set_updated_at ON discord_users;
CREATE TRIGGER set_updated_at
BEFORE UPDATE ON discord_users
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_updated_at();

CREATE TABLE IF NOT EXISTS discord_tokens (
  access_token VARCHAR PRIMARY KEY,
  refresh_token VARCHAR NOT NULL,
  scopes VARCHAR NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  discord_user_id VARCHAR NOT NULL,
  CONSTRAINT fk_discord_user_id FOREIGN KEY(discord_user_id) REFERENCES discord_users(id)
);
DROP TRIGGER IF EXISTS set_updated_at ON discord_tokens;
CREATE TRIGGER set_updated_at
BEFORE UPDATE ON discord_tokens
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_updated_at();

CREATE TABLE IF NOT EXISTS plex_users (
  id BIGINT PRIMARY KEY,
  username VARCHAR NOT NULL, 
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  discord_user_id VARCHAR NOT NULL,
  is_subscriber BOOLEAN NOT NULL,
  CONSTRAINT fk_discord_user_id FOREIGN KEY(discord_user_id) REFERENCES discord_users(id)
);
DROP TRIGGER IF EXISTS set_updated_at ON plex_users;
CREATE TRIGGER set_updated_at
BEFORE UPDATE ON plex_users
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_updated_at();

CREATE TABLE IF NOT EXISTS plex_tokens (
  access_token VARCHAR PRIMARY KEY,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  plex_user_id BIGINT NOT NULL,
  CONSTRAINT fk_plex_user_id FOREIGN KEY(plex_user_id) REFERENCES plex_users(id)
);
DROP TRIGGER IF EXISTS set_updated_at ON plex_tokens;
CREATE TRIGGER set_updated_at
BEFORE UPDATE ON plex_tokens
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_updated_at();

DROP TABLE IF EXISTS __diesel_schema_migrations;
