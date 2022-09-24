CREATE TABLE
    "public"."admin" (
        "character_id" int8 NOT NULL,
        "role" varchar NOT NULL,
        "granted_at" int8 NOT NULL,
        "granted_by_id" int8,
        PRIMARY KEY ("character_id"),
        CONSTRAINT "fk_admin_character_role" FOREIGN KEY ("character_id") REFERENCES "public"."character" ("id"),
        CONSTRAINT "fk_admin_granted_by_id" FOREIGN KEY ("granted_by_id") REFERENCES "public"."character" ("id")
    );
