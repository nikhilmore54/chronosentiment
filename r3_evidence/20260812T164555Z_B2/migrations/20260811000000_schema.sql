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

CREATE TABLE IF NOT EXISTS knowledge_outcomes (
    id UUID PRIMARY KEY,

    decision_id UUID NOT NULL,
    strategy_id UUID NOT NULL,
    instrument_id UUID REFERENCES instruments(id),

    evaluation_timestamp TIMESTAMPTZ NOT NULL,
    horizon VARCHAR(20) NOT NULL,
    horizon_expiry_timestamp TIMESTAMPTZ NOT NULL,
    observation_end_timestamp TIMESTAMPTZ NOT NULL,

    entry_reached BOOLEAN NOT NULL,
    target_hit BOOLEAN NOT NULL,
    stop_hit BOOLEAN NOT NULL,

    exit_reason VARCHAR(50) NOT NULL,

    outcome_return DOUBLE PRECISION NOT NULL,
    mfe DOUBLE PRECISION NOT NULL,
    mae DOUBLE PRECISION NOT NULL,
    drawdown DOUBLE PRECISION NOT NULL,

    metadata_json JSONB NOT NULL,
    outcome_json JSONB NOT NULL,

    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_know_outcomes_instrument ON knowledge_outcomes(instrument_id);
CREATE INDEX IF NOT EXISTS idx_know_outcomes_eval_time ON knowledge_outcomes(evaluation_timestamp);
CREATE INDEX IF NOT EXISTS idx_know_outcomes_horizon ON knowledge_outcomes(horizon);
CREATE INDEX IF NOT EXISTS idx_know_outcomes_decision ON knowledge_outcomes(decision_id);
CREATE INDEX IF NOT EXISTS idx_know_outcomes_strategy ON knowledge_outcomes(strategy_id);
CREATE INDEX IF NOT EXISTS idx_know_outcomes_exit ON knowledge_outcomes(exit_reason);

CREATE TABLE IF NOT EXISTS knowledge_decisions (
    id UUID PRIMARY KEY,
    instrument_id UUID REFERENCES instruments(id),
    evaluation_timestamp TIMESTAMPTZ NOT NULL,
    opportunity VARCHAR(50) NOT NULL,
    metadata_json JSONB NOT NULL,
    decision_json JSONB NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_know_decisions_time ON knowledge_decisions(evaluation_timestamp);
CREATE INDEX IF NOT EXISTS idx_know_decisions_inst ON knowledge_decisions(instrument_id);

CREATE TABLE IF NOT EXISTS knowledge_strategies (
    id UUID PRIMARY KEY,
    decision_id UUID NOT NULL,
    expected_horizon VARCHAR(50) NOT NULL,
    metadata_json JSONB NOT NULL,
    strategy_json JSONB NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_know_strategies_decision ON knowledge_strategies(decision_id);
