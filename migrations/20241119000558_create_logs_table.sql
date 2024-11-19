-- sqlx database drop/ccreate
-- sqlx migrate run
CREATE TABLE IF NOT EXISTS author (
  id uuid primary key,
  name text,
);
CREATE TABLE IF NOT EXISTS project (
  id uuid primary key,
  author uuid references author,
  created integer,
  version integer, -- update version of this entry
  revision integer, -- code revision when updated
  name text,
);
CREATE TABLE IF NOT EXISTS log (
  id uuid primary key,
  project uuid references project,
  author uuid references author,
  version integer, -- update version of this entry
  revision integer, -- code revision when updated
  created integer,
  text text
);
