-- This file should undo anything in `up.sql`
DROP TABLE users;
DROP INDEX actors_unique_idx_uri;
DROP INDEX actors_unique_idx_username_domain;
DROP TABLE actors;