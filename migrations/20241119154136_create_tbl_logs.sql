-- Add migration script here
CREATE TABLE IF NOT EXISTS log (
  id blob       primary key         not null,
  project blob  references project  not null,
  author blob   references author   not null,
  version integer, -- update version of this entry
  revision integer, -- code revision when updated
  created integer                   not null,
  text text                         not null
)
