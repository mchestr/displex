-- Your SQL goes here
CREATE TABLE plex_tokens (
  id VARCHAR PRIMARY KEY,
  username VARCHAR NOT NULL,
  access_token VARCHAR NOT NULL
);
