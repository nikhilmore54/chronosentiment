-- ChronoSentiment Phase 1: Canonical Observation Lake Schema

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
-- Uncomment when pgvector is installed
-- CREATE EXTENSION IF NOT EXISTS vector;

-- 1. Instruments (Master)
-- Core identifiable things in the world (Equities, Macro Indicators)
CREATE TABLE instruments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    exchange VARCHAR(50) NOT NULL, -- e.g., 'NSE', 'BSE', 'FRED'
    display_symbol VARCHAR(50) NOT NULL, -- e.g., 'RELIANCE', 'CPI'
    provider_ids JSONB DEFAULT '{}'::jsonb, -- e.g. {"kite_token": "738561"}
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX idx_instruments_symbol ON instruments(exchange, display_symbol);

-- 2. Relationships (Between Instruments)
CREATE TABLE instrument_relationships (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_instrument_id UUID NOT NULL REFERENCES instruments(id),
    target_instrument_id UUID NOT NULL REFERENCES instruments(id),
    relationship_type VARCHAR(50) NOT NULL,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(source_instrument_id, target_instrument_id, relationship_type)
);

-- 3. Observations
-- The immutable bedrock: "Something happened in the real world"
CREATE TABLE observations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    observation_type VARCHAR(50) NOT NULL, -- e.g., 'PriceAction', 'NewsSentiment', 'MacroRelease', 'EarningsReport'
    
    -- Temporal Dimensions
    observed_at TIMESTAMPTZ NOT NULL,
    effective_from TIMESTAMPTZ,
    effective_to TIMESTAMPTZ,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    instrument_id UUID REFERENCES instruments(id),
    
    -- Core payload
    raw_payload JSONB DEFAULT '{}'::jsonb, -- The exact untouched payload from the provider
    normalized_payload JSONB DEFAULT '{}'::jsonb, -- Structured data (e.g., OHLCV)
    
    -- Knowledge Confidence Metadata
    confidence_score DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    freshness_at DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    quality_score DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    source_name VARCHAR(255) NOT NULL, -- e.g., 'FRED', 'Kite', 'SEC'
    coverage VARCHAR(50) NOT NULL DEFAULT 'Complete',
    consistency_score DOUBLE PRECISION, -- Null if not yet cross-checked
    
    provenance_hash VARCHAR(64) NOT NULL,
    schema_version INT NOT NULL DEFAULT 1
);
CREATE INDEX idx_observations_type ON observations(observation_type);
CREATE INDEX idx_observations_observed_at ON observations(observed_at);
CREATE INDEX idx_observations_instrument ON observations(instrument_id);

-- 4. Metrics
-- Quantified observations or derived indicators
CREATE TABLE metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    metric_name VARCHAR(100) NOT NULL, -- e.g., 'RSI_14', 'PE_Ratio'
    entity_id UUID NOT NULL REFERENCES entities(id),
    calculated_at TIMESTAMPTZ NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    derived_from_observations UUID[], -- Array of observation IDs that fed this metric
    metadata JSONB DEFAULT '{}'::jsonb,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_metrics_entity_name ON metrics(entity_id, metric_name);
CREATE INDEX idx_metrics_calculated_at ON metrics(calculated_at);

-- 5. Research Sessions
-- A top-level container grouping related observations, evidence, hypotheses, and decisions
CREATE TABLE research_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'Open', -- 'Open', 'Closed'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at TIMESTAMPTZ
);

-- 6. Hypotheses
-- Interpretations of evidence over a horizon
CREATE TABLE hypotheses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID REFERENCES research_sessions(id),
    author VARCHAR(100) NOT NULL, -- 'Human' or 'Coralys'
    target_entity_id UUID NOT NULL REFERENCES entities(id),
    hypothesis_statement TEXT NOT NULL,
    expected_horizon_start TIMESTAMPTZ,
    expected_horizon_end TIMESTAMPTZ,
    status VARCHAR(50) NOT NULL DEFAULT 'Active', -- 'Active', 'Resolved', 'Invalidated'
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 7. Evidence Linkage
-- Contextual interpretation mapping an observation to a hypothesis
CREATE TABLE evidence (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    hypothesis_id UUID NOT NULL REFERENCES hypotheses(id),
    observation_id UUID NOT NULL REFERENCES observations(id),
    stance VARCHAR(50) NOT NULL, -- 'Supports', 'Contradicts'
    weight DOUBLE PRECISION NOT NULL DEFAULT 1.0, -- Strength of evidence
    confidence DOUBLE PRECISION NOT NULL DEFAULT 1.0, -- Confidence in this specific interpretation
    recency_interval INTERVAL, -- e.g., '3 days'
    rationale TEXT, -- Why does this observation support/contradict?
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(hypothesis_id, observation_id)
);

-- 8. Decisions
-- Selections among competing hypotheses
CREATE TABLE decisions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID REFERENCES research_sessions(id),
    hypothesis_id UUID NOT NULL REFERENCES hypotheses(id),
    decision_action VARCHAR(50) NOT NULL, -- e.g., 'Buy', 'Sell', 'Hold', 'Monitor'
    confidence DOUBLE PRECISION NOT NULL,
    expected_outcome TEXT,
    decided_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 9. Outcomes & Learning (Journal)
-- Validations of reasoning
CREATE TABLE outcomes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    decision_id UUID NOT NULL REFERENCES decisions(id),
    actual_outcome TEXT NOT NULL,
    success_score DOUBLE PRECISION NOT NULL, -- -1.0 (Total Failure) to 1.0 (Total Success)
    lessons_learned TEXT,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 10. Knowledge Assessments (Phase 5.1)
CREATE TABLE knowledge_assessments (
    id UUID PRIMARY KEY, -- Maps to ArtifactMetadata.artifact_id
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
CREATE INDEX idx_know_assess_sig ON knowledge_assessments(signature_hash);
CREATE INDEX idx_know_assess_time ON knowledge_assessments(evaluation_timestamp);
