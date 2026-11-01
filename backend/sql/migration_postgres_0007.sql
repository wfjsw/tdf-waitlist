-- Each waitlist's SRP rule: which wallet division is read, the amount that counts, and how long a
-- payment covers. The values are stored per waitlist (edited on the SRP wallet page), not read from
-- code constants; existing waitlists get the constants' values. Runs after 0005 (and 0006 where it
-- applied). Re-running it is a no-op.
DO $$
BEGIN
    IF to_regclass('public.srp_waitlist') IS NULL THEN
        RETURN;
    END IF;
    ALTER TABLE "public"."srp_waitlist"
        ADD COLUMN IF NOT EXISTS "division" int4 NOT NULL DEFAULT 1,
        ADD COLUMN IF NOT EXISTS "min_amount" int8 NOT NULL DEFAULT 50000000,
        ADD COLUMN IF NOT EXISTS "coverage_hours" int4 NOT NULL DEFAULT 24;
END
$$;

-- Mirrors core::srp::validate_rule (1-7 divisions, 30-day maximum coverage)
DO $$
BEGIN
    IF to_regclass('public.srp_waitlist') IS NULL THEN
        RETURN;
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'srp_waitlist_division_range'
    ) THEN
        ALTER TABLE "public"."srp_waitlist"
            ADD CONSTRAINT "srp_waitlist_division_range" CHECK ("division" BETWEEN 1 AND 7);
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'srp_waitlist_min_amount_positive'
    ) THEN
        ALTER TABLE "public"."srp_waitlist"
            ADD CONSTRAINT "srp_waitlist_min_amount_positive" CHECK ("min_amount" >= 0);
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'srp_waitlist_coverage_range'
    ) THEN
        ALTER TABLE "public"."srp_waitlist"
            ADD CONSTRAINT "srp_waitlist_coverage_range" CHECK ("coverage_hours" BETWEEN 1 AND 720);
    END IF;
END
$$;
