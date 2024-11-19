-- Add migration script here

CREATE TABLE IF NOT EXISTS projects (
  id uuid primary key,
  name text,
  created integer
);

CREATE TABLE IF NOT EXISTS logs (
  id uuid primary key,
  project_id uuid,
  author_id uuid,
  created integer,
  text text
);
