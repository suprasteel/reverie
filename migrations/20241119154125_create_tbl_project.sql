-- Add migration script here
CREATE TABLE IF NOT EXISTS project (
  id blob         primary key       not null,
  author blob     references author not null,
  created integer                   not null,
  version integer                   not null, -- update version of this entry
  revision integer, -- code revision when updated
  name text                         not null
)
