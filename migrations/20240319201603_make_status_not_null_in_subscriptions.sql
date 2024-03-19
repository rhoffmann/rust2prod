-- transaction
BEGIN;
-- backfill 'status'
UPDATE subscriptions
SET status = 'confirmed'
WHERE status IS NULL;
-- make 'status' NOT NULL
ALTER TABLE subscriptions
    ALTER COLUMN status
        SET NOT NULL;
COMMIT;