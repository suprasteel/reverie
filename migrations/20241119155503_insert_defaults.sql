-- Add migration script here
INSERT INTO author(id,name) VALUES (0, 'me');
INSERT INTO project(id,author,created,version,revision,name) VALUES (0, 0, 0, 1, 1, 'default');
