-- Your SQL goes here
CREATE TABLE discord_users (
  id VARCHAR PRIMARY KEY,
  username VARCHAR NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
SELECT diesel_manage_updated_at('discord_users');

CREATE TABLE discord_tokens (
  access_token VARCHAR PRIMARY KEY,
  refresh_token VARCHAR NOT NULL,
  scopes VARCHAR NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  discord_user_id VARCHAR NOT NULL,
  CONSTRAINT fk_discord_user_id FOREIGN KEY(discord_user_id) REFERENCES discord_users(id)
);
SELECT diesel_manage_updated_at('discord_tokens');

CREATE TABLE plex_users (
  id VARCHAR PRIMARY KEY,
  username VARCHAR NOT NULL, 
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  discord_user_id VARCHAR NOT NULL,
  CONSTRAINT fk_discord_user_id FOREIGN KEY(discord_user_id) REFERENCES discord_users(id)
);
SELECT diesel_manage_updated_at('plex_users');

CREATE TABLE plex_tokens (
  access_token VARCHAR PRIMARY KEY,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  plex_user_id VARCHAR NOT NULL,
  CONSTRAINT fk_plex_user_id FOREIGN KEY(plex_user_id) REFERENCES plex_users(id)
);
SELECT diesel_manage_updated_at('plex_tokens');
