CREATE TABLE
    "public"."announcement" (
        "id" int8 NOT NULL,
        "message" text NOT NULL,
        "character_id" int8 NOT NULL,
        "created_at" int8 NOT NULL,
        PRIMARY KEY ("id"),
        CONSTRAINT "fk_announcement_created_by_character_id" FOREIGN KEY ("character_id") REFERENCES "public"."character" ("id")
    );
