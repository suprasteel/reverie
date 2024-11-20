-- Add migration script here
CREATE TABLE IF NOT EXISTS author
(
  id    uuid primary key  not null,
  name  text              not null
)
