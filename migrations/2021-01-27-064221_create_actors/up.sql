-- Your SQL goes here

CREATE TABLE "actors" (
  "id" BIGSERIAL PRIMARY KEY,
  "uri" VARCHAR NOT NULL,
  "url" VARCHAR,
  "kind" VARCHAR NOT NULL DEFAULT 'Person',

  "username" VARCHAR NOT NULL,
  "domain" VARCHAR NOT NULL,
  "name" VARCHAR NOT NULL DEFAULT '',
  "summary" TEXT NOT NULL DEFAULT '',
  "avatar_url" VARCHAR NOT NULL DEFAULT '',

  "inbox_uri" VARCHAR NOT NULL,
  "outbox_uri" VARCHAR NOT NULL,
  "followers_uri" VARCHAR,
  "following_uri" VARCHAR,

  public_key_pem TEXT NOT NULL,

  "created_at" TIMESTAMP NOT NULL,
  "updated_at" TIMESTAMP,

  lang VARCHAR NOT NULL DEFAULT 'und',
  is_locked BOOLEAN NOT NULL DEFAULT FALSE,
  is_suspended BOOLEAN NOT NULL DEFAULT FALSE,
  is_silenced BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE UNIQUE INDEX actors_unique_idx_username_domain ON actors (lower(username), lower(domain));
CREATE UNIQUE INDEX actors_unique_idx_uri ON actors (lower(uri));

CREATE TABLE users (
  actor_id BIGINT PRIMARY KEY,
  email VARCHAR,
  is_email_verified BOOLEAN NOT NULL DEFAULT TRUE,
  password_hash VARCHAR,
  private_key_pem TEXT NOT NULL,
  register_ip VARCHAR,
  last_login_ip VARCHAR,
  CONSTRAINT fk_actor
    FOREIGN KEY(actor_id) 
	    REFERENCES actors(id)
      ON DELETE CASCADE
      ON UPDATE CASCADE
);