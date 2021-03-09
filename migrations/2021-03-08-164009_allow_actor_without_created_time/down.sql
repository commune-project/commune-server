-- This file should undo anything in `up.sql`
ALTER TABLE "actors" ALTER COLUMN "created_at" SET NOT NULL;