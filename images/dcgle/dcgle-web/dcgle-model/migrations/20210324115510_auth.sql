-- Add migration script here

CREATE TABLE user (
  id TEXT,
  name TEXT,
  kind TEXT
);

CREATE TABLE auth (
  id TEXT,
  name TEXT,
  value TEXT
);
