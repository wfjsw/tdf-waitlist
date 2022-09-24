CREATE TABLE
    "public"."badge" (
        "id" serial8,
        "name" varchar NOT NULL,
        "exclude_badge_id" int8,
        PRIMARY KEY ("id"),
        CONSTRAINT "fk_badge_exclude_badge" FOREIGN KEY ("exclude_badge_id") REFERENCES "public"."badge" ("id") ON DELETE
        SET
            NULL,
            CONSTRAINT "uniq_badge_name" UNIQUE ("name")
    );

CREATE TABLE
    "public"."badge_assignment" (
        "characterId" int8 NOT NULL,
        "badgeId" int8 NOT NULL,
        "grantedById" int8,
        "grantedAt" int8 NOT NULL,
        PRIMARY KEY ("characterId", "badgeId"),
        CONSTRAINT "fk_badge_assignment_characterId" FOREIGN KEY ("characterId") REFERENCES "public"."character" ("id"),
        CONSTRAINT "fk_badge_assignment_badgeId" FOREIGN KEY ("badgeId") REFERENCES "public"."badge" ("id") ON DELETE CASCADE,
        CONSTRAINT "fk_badge_assignment_grantedById" FOREIGN KEY ("grantedById") REFERENCES "public"."character" ("id")
    );

INSERT INTO badge (name) VALUES ('BASTION');

INSERT INTO badge (name) VALUES ('LOGI');

INSERT INTO badge (name) VALUES ('RETIRED-LOGI');

INSERT INTO badge (name) VALUES ('WEB');
