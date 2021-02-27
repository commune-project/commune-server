-- Your SQL goes here
CREATE TABLE "follows" (
    "follower_id" bigint,
    "following_id" bigint,
    "created_at" timestamp NOT NULL,
    "updated_at" timestamp,
    "role" varchar NOT NULL,
    PRIMARY KEY ("follower_id", "following_id"),
    CONSTRAINT "fk_follows_following" FOREIGN KEY ("following_id") REFERENCES "actors"("id"),
    CONSTRAINT "fk_follows_actor" FOREIGN KEY ("following_id") REFERENCES "actors"("id"),
    CONSTRAINT "fk_follows_followers" FOREIGN KEY ("follower_id") REFERENCES "actors"("id"),
    CONSTRAINT "fk_follows_follower" FOREIGN KEY ("follower_id") REFERENCES "actors"("id")
);