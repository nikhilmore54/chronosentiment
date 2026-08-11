CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS instruments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    exchange VARCHAR(50) NOT NULL,
    display_symbol VARCHAR(50) NOT NULL,
    provider_ids JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_instruments_symbol ON instruments(exchange, display_symbol);

CREATE TABLE IF NOT EXISTS knowledge_assessments (
    id UUID PRIMARY KEY,
    instrument_id UUID REFERENCES instruments(id),
    evaluation_timestamp TIMESTAMPTZ NOT NULL,
    
    market_assessment_json JSONB DEFAULT '{}'::jsonb,
    sector_assessment_json JSONB DEFAULT '{}'::jsonb,
    instrument_assessment_json JSONB DEFAULT '{}'::jsonb,
    macro_assessment_json JSONB DEFAULT '{}'::jsonb,
    
    signature JSONB NOT NULL,
    signature_hash TEXT NOT NULL,
    
    metadata_json JSONB NOT NULL,
    profile_json JSONB NOT NULL,
    
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_know_assess_sig ON knowledge_assessments(signature_hash);
CREATE INDEX IF NOT EXISTS idx_know_assess_time ON knowledge_assessments(evaluation_timestamp);
