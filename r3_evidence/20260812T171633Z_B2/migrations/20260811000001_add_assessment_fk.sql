BEGIN;

-- Add assessment_id column to knowledge_decisions
ALTER TABLE knowledge_decisions
    ADD COLUMN assessment_id UUID NOT NULL;

-- Add foreign key constraint linking to knowledge_assessments
ALTER TABLE knowledge_decisions
    ADD CONSTRAINT fk_decision_assessment
    FOREIGN KEY (assessment_id)
    REFERENCES knowledge_assessments(id)
    ON DELETE RESTRICT;

COMMIT;
