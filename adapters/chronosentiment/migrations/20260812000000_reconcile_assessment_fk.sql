-- Migration to relax NOT NULL constraint on assessment_id in knowledge_decisions
-- Allows B2 population to succeed before R-layer backfill and constraint enforcement.
BEGIN;
ALTER TABLE knowledge_decisions ALTER COLUMN assessment_id DROP NOT NULL;
COMMIT;
