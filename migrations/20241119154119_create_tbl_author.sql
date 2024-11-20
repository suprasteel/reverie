-- Add migration script here
CREATE TABLE IF NOT EXISTS author
(
  id    blob primary key  not null,
  name  text              not null
)
