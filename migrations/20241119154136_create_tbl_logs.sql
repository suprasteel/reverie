-- Add migration script here
CREATE TABLE IF NOT EXISTS log (
  id uuid primary key,
  project uuid references project,
  author uuid references author,
  version integer, -- update version of this entry
  revision integer, -- code revision when updated
  created integer,
  text text
)
