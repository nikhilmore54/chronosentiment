--
-- PostgreSQL database dump
--

-- Dumped from database version 14.17 (Homebrew)
-- Dumped by pg_dump version 14.17 (Homebrew)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: uuid-ossp; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;


--
-- Name: EXTENSION "uuid-ossp"; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION "uuid-ossp" IS 'generate universally unique identifiers (UUIDs)';


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: instruments; Type: TABLE; Schema: public; Owner: nikhil
--

CREATE TABLE public.instruments (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    exchange character varying(50) NOT NULL,
    display_symbol character varying(50) NOT NULL,
    provider_ids jsonb DEFAULT '{}'::jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.instruments OWNER TO nikhil;

--
-- Name: knowledge_assessments; Type: TABLE; Schema: public; Owner: nikhil
--

CREATE TABLE public.knowledge_assessments (
    id uuid NOT NULL,
    instrument_id uuid,
    evaluation_timestamp timestamp with time zone NOT NULL,
    market_assessment_json jsonb DEFAULT '{}'::jsonb,
    sector_assessment_json jsonb DEFAULT '{}'::jsonb,
    instrument_assessment_json jsonb DEFAULT '{}'::jsonb,
    macro_assessment_json jsonb DEFAULT '{}'::jsonb,
    signature jsonb NOT NULL,
    signature_hash text NOT NULL,
    metadata_json jsonb NOT NULL,
    profile_json jsonb NOT NULL,
    recorded_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.knowledge_assessments OWNER TO nikhil;

--
-- Name: knowledge_decisions; Type: TABLE; Schema: public; Owner: nikhil
--

CREATE TABLE public.knowledge_decisions (
    id uuid NOT NULL,
    instrument_id uuid,
    evaluation_timestamp timestamp with time zone NOT NULL,
    opportunity character varying(50) NOT NULL,
    metadata_json jsonb NOT NULL,
    decision_json jsonb NOT NULL,
    recorded_at timestamp with time zone DEFAULT now() NOT NULL,
    assessment_id uuid NOT NULL
);


ALTER TABLE public.knowledge_decisions OWNER TO nikhil;

--
-- Name: knowledge_outcomes; Type: TABLE; Schema: public; Owner: nikhil
--

CREATE TABLE public.knowledge_outcomes (
    id uuid NOT NULL,
    decision_id uuid NOT NULL,
    strategy_id uuid NOT NULL,
    instrument_id uuid,
    evaluation_timestamp timestamp with time zone NOT NULL,
    horizon character varying(20) NOT NULL,
    horizon_expiry_timestamp timestamp with time zone NOT NULL,
    observation_end_timestamp timestamp with time zone NOT NULL,
    entry_reached boolean NOT NULL,
    target_hit boolean NOT NULL,
    stop_hit boolean NOT NULL,
    exit_reason character varying(50) NOT NULL,
    outcome_return double precision NOT NULL,
    mfe double precision NOT NULL,
    mae double precision NOT NULL,
    drawdown double precision NOT NULL,
    metadata_json jsonb NOT NULL,
    outcome_json jsonb NOT NULL,
    recorded_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.knowledge_outcomes OWNER TO nikhil;

--
-- Name: knowledge_strategies; Type: TABLE; Schema: public; Owner: nikhil
--

CREATE TABLE public.knowledge_strategies (
    id uuid NOT NULL,
    decision_id uuid NOT NULL,
    expected_horizon character varying(50) NOT NULL,
    metadata_json jsonb NOT NULL,
    strategy_json jsonb NOT NULL,
    recorded_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.knowledge_strategies OWNER TO nikhil;

--
-- Name: instruments instruments_pkey; Type: CONSTRAINT; Schema: public; Owner: nikhil
--

ALTER TABLE ONLY public.instruments
    ADD CONSTRAINT instruments_pkey PRIMARY KEY (id);


--
-- Name: knowledge_assessments knowledge_assessments_pkey; Type: CONSTRAINT; Schema: public; Owner: nikhil
--

ALTER TABLE ONLY public.knowledge_assessments
    ADD CONSTRAINT knowledge_assessments_pkey PRIMARY KEY (id);


--
-- Name: knowledge_decisions knowledge_decisions_pkey; Type: CONSTRAINT; Schema: public; Owner: nikhil
--

ALTER TABLE ONLY public.knowledge_decisions
    ADD CONSTRAINT knowledge_decisions_pkey PRIMARY KEY (id);


--
-- Name: knowledge_outcomes knowledge_outcomes_pkey; Type: CONSTRAINT; Schema: public; Owner: nikhil
--

ALTER TABLE ONLY public.knowledge_outcomes
    ADD CONSTRAINT knowledge_outcomes_pkey PRIMARY KEY (id);


--
-- Name: knowledge_strategies knowledge_strategies_pkey; Type: CONSTRAINT; Schema: public; Owner: nikhil
--

ALTER TABLE ONLY public.knowledge_strategies
    ADD CONSTRAINT knowledge_strategies_pkey PRIMARY KEY (id);


--
-- Name: idx_instruments_symbol; Type: INDEX; Schema: public; Owner: nikhil
--

CREATE UNIQUE INDEX idx_instruments_symbol ON public.instruments USING btree (exchange, display_symbol);


--
-- Name: idx_know_assess_sig; Type: INDEX; Schema: public; Owner: nikhil
--

CREATE INDEX idx_know_assess_sig ON public.knowledge_assessments USING btree (signature_hash);


--
-- Name: idx_know_assess_time; Type: INDEX; Schema: public; Owner: nikhil
--

CREATE INDEX idx_know_assess_time ON public.knowledge_assessments USING btree (evaluation_timestamp);


--
-- Name: idx_know_decisions_inst; Type: INDEX; Schema: public; Owner: nikhil
--

CREATE INDEX idx_know_decisions_inst ON public.knowledge_decisions USING btree (instrument_id);


--
-- Name: idx_know_decisions_time; Type: INDEX; Schema: public; Owner: nikhil
--

CREATE INDEX idx_know_decisions_time ON public.knowledge_decisions USING btree (evaluation_timestamp);


--
-- Name: idx_know_outcomes_decision; Type: INDEX; Schema: public; Owner: nikhil
--

CREATE INDEX idx_know_outcomes_decision ON public.knowledge_outcomes USING btree (decision_id);


--
-- Name: idx_know_outcomes_eval_time; Type: INDEX; Schema: public; Owner: nikhil
--

CREATE INDEX idx_know_outcomes_eval_time ON public.knowledge_outcomes USING btree (evaluation_timestamp);


--
-- Name: idx_know_outcomes_exit; Type: INDEX; Schema: public; Owner: nikhil
--

CREATE INDEX idx_know_outcomes_exit ON public.knowledge_outcomes USING btree (exit_reason);


--
-- Name: idx_know_outcomes_horizon; Type: INDEX; Schema: public; Owner: nikhil
--

CREATE INDEX idx_know_outcomes_horizon ON public.knowledge_outcomes USING btree (horizon);


--
-- Name: idx_know_outcomes_instrument; Type: INDEX; Schema: public; Owner: nikhil
--

CREATE INDEX idx_know_outcomes_instrument ON public.knowledge_outcomes USING btree (instrument_id);


--
-- Name: idx_know_outcomes_strategy; Type: INDEX; Schema: public; Owner: nikhil
--

CREATE INDEX idx_know_outcomes_strategy ON public.knowledge_outcomes USING btree (strategy_id);


--
-- Name: idx_know_strategies_decision; Type: INDEX; Schema: public; Owner: nikhil
--

CREATE INDEX idx_know_strategies_decision ON public.knowledge_strategies USING btree (decision_id);


--
-- Name: knowledge_decisions fk_decision_assessment; Type: FK CONSTRAINT; Schema: public; Owner: nikhil
--

ALTER TABLE ONLY public.knowledge_decisions
    ADD CONSTRAINT fk_decision_assessment FOREIGN KEY (assessment_id) REFERENCES public.knowledge_assessments(id) ON DELETE RESTRICT;


--
-- Name: knowledge_assessments knowledge_assessments_instrument_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nikhil
--

ALTER TABLE ONLY public.knowledge_assessments
    ADD CONSTRAINT knowledge_assessments_instrument_id_fkey FOREIGN KEY (instrument_id) REFERENCES public.instruments(id);


--
-- Name: knowledge_decisions knowledge_decisions_instrument_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nikhil
--

ALTER TABLE ONLY public.knowledge_decisions
    ADD CONSTRAINT knowledge_decisions_instrument_id_fkey FOREIGN KEY (instrument_id) REFERENCES public.instruments(id);


--
-- Name: knowledge_outcomes knowledge_outcomes_instrument_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nikhil
--

ALTER TABLE ONLY public.knowledge_outcomes
    ADD CONSTRAINT knowledge_outcomes_instrument_id_fkey FOREIGN KEY (instrument_id) REFERENCES public.instruments(id);


--
-- PostgreSQL database dump complete
--

