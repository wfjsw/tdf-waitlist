-- Upgrades a database that ran the first version of 0005, which keyed SRP wallets by corp, to the
-- current shape: a wallet per character, and a wallet character per waitlist. Waitlists that
-- pointed at a since-disconnected corp keep the corp (and show nothing) until a wallet for it is
-- connected again. Does nothing when 0005 was never applied or already ran in its current shape.
-- Also finishes a 0005 that only got as far as its first table.
DO $$
DECLARE
    pk_name text;
BEGIN
    -- Nothing to upgrade when 0005 was never applied, or already ran in its current shape. The
    -- old srp_waitlist is the one without a character_id column (the old srp_connection already
    -- had one, under its corporation primary key).
    IF to_regclass('public.srp_connection') IS NOT NULL
       AND (
           to_regclass('public.srp_waitlist') IS NULL
           OR NOT EXISTS (
               SELECT 1 FROM information_schema.columns
               WHERE table_schema = 'public' AND table_name = 'srp_waitlist'
                 AND column_name = 'character_id'
           )
       )
    THEN
        -- Index names are schema-wide, and renaming a table keeps its primary key's name: move
        -- the old key aside instead of relying on its default name
        SELECT conname INTO pk_name FROM pg_constraint
        WHERE conrelid = 'public.srp_connection'::regclass AND contype = 'p';
        EXECUTE format(
            'ALTER TABLE public.srp_connection RENAME CONSTRAINT %I TO srp_connection_0005_pkey',
            pk_name
        );
        ALTER TABLE "public"."srp_connection" RENAME TO "srp_connection_0005";
        -- 0005 can have been re-run after an earlier upgrade attempt created this index
        ALTER INDEX IF EXISTS "public"."srp_connection_corporation"
            RENAME TO "srp_connection_0005_corporation";

        IF to_regclass('public.srp_waitlist') IS NOT NULL THEN
            SELECT conname INTO pk_name FROM pg_constraint
            WHERE conrelid = 'public.srp_waitlist'::regclass AND contype = 'p';
            EXECUTE format(
                'ALTER TABLE public.srp_waitlist RENAME CONSTRAINT %I TO srp_waitlist_0005_pkey',
                pk_name
            );
            ALTER TABLE "public"."srp_waitlist" RENAME TO "srp_waitlist_0005";
        END IF;

        CREATE TABLE
            "public"."srp_connection" (
                "character_id" int8 NOT NULL,
                "corporation_id" int8 NOT NULL,
                "corporation_name" varchar NOT NULL,
                "refresh_token" text NOT NULL,
                "connected_by" int8 NOT NULL,
                "connected_at" int8 NOT NULL,
                PRIMARY KEY ("character_id"),
                CONSTRAINT "fk_srp_connection_character" FOREIGN KEY ("character_id") REFERENCES "public"."character" ("id"),
                CONSTRAINT "fk_srp_connection_connected_by" FOREIGN KEY ("connected_by") REFERENCES "public"."character" ("id")
            );

        CREATE INDEX "srp_connection_corporation" ON "public"."srp_connection" ("corporation_id");

        -- The old primary key was the corp, so a character that moved corp and was connected again
        -- has a row per corp: keep the newest one, and its token
        INSERT INTO "public"."srp_connection"
            (character_id, corporation_id, corporation_name, refresh_token, connected_by, connected_at)
            SELECT DISTINCT ON (character_id)
                character_id, corporation_id, corporation_name, refresh_token, connected_by, connected_at
            FROM "public"."srp_connection_0005"
            ORDER BY character_id, connected_at DESC;

        CREATE TABLE
            "public"."srp_waitlist" (
                "waitlist_id" int8 NOT NULL,
                "character_id" int8 NULL,
                "corporation_id" int8 NULL,
                "set_by" int8 NOT NULL,
                "set_at" int8 NOT NULL,
                PRIMARY KEY ("waitlist_id"),
                CONSTRAINT "fk_srp_waitlist_waitlist" FOREIGN KEY ("waitlist_id") REFERENCES "public"."waitlist" ("id") ON DELETE CASCADE,
                CONSTRAINT "fk_srp_waitlist_character" FOREIGN KEY ("character_id") REFERENCES "public"."character" ("id"),
                CONSTRAINT "fk_srp_waitlist_set_by" FOREIGN KEY ("set_by") REFERENCES "public"."character" ("id")
            );

        -- A waitlist pointing at a corp with no connected wallet has no character to point at:
        -- keep the corp in corporation_id, which it falls back to until a wallet of that corp is
        -- connected. Waitlists that do have a character follow that account from now on.
        IF to_regclass('public.srp_waitlist_0005') IS NOT NULL THEN
            INSERT INTO "public"."srp_waitlist" (waitlist_id, character_id, corporation_id, set_by, set_at)
                SELECT w.waitlist_id, c.character_id,
                    CASE WHEN c.character_id IS NULL THEN w.corporation_id END, w.set_by, w.set_at
                FROM "public"."srp_waitlist_0005" w
                LEFT JOIN "public"."srp_connection_0005" c USING (corporation_id);

            DROP TABLE "public"."srp_waitlist_0005";
        END IF;

        DROP TABLE "public"."srp_connection_0005";
    END IF;
END $$;
