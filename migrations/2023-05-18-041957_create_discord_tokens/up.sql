-- Your SQL goes here
CREATE TABLE discord_tokens (
  id VARCHAR PRIMARY KEY,
  username VARCHAR NOT NULL,
  access_token VARCHAR NOT NULL,
  token_type VARCHAR NOT NULL,
  refresh_token VARCHAR NOT NULL,
  scopes VARCHAR NOT NULL
);
