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
-- Name: _sqlx_test; Type: SCHEMA; Schema: -; Owner: nikhil
--

CREATE SCHEMA _sqlx_test;


ALTER SCHEMA _sqlx_test OWNER TO nikhil;

--
-- Name: hdb_catalog; Type: SCHEMA; Schema: -; Owner: postgres
--

CREATE SCHEMA hdb_catalog;


ALTER SCHEMA hdb_catalog OWNER TO postgres;

--
-- Name: hstore; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS hstore WITH SCHEMA public;


--
-- Name: EXTENSION hstore; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION hstore IS 'data type for storing sets of (key, value) pairs';


--
-- Name: pgcrypto; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;


--
-- Name: EXTENSION pgcrypto; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION pgcrypto IS 'cryptographic functions';


--
-- Name: uuid-ossp; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;


--
-- Name: EXTENSION "uuid-ossp"; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION "uuid-ossp" IS 'generate universally unique identifiers (UUIDs)';


--
-- Name: gen_hasura_uuid(); Type: FUNCTION; Schema: hdb_catalog; Owner: postgres
--

CREATE FUNCTION hdb_catalog.gen_hasura_uuid() RETURNS uuid
    LANGUAGE sql
    AS $$select gen_random_uuid()$$;


ALTER FUNCTION hdb_catalog.gen_hasura_uuid() OWNER TO postgres;

--
-- Name: database_ids; Type: SEQUENCE; Schema: _sqlx_test; Owner: nikhil
--

CREATE SEQUENCE _sqlx_test.database_ids
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE _sqlx_test.database_ids OWNER TO nikhil;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: databases; Type: TABLE; Schema: _sqlx_test; Owner: nikhil
--

CREATE TABLE _sqlx_test.databases (
    db_name text NOT NULL,
    test_path text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE _sqlx_test.databases OWNER TO nikhil;

--
-- Name: hdb_action_log; Type: TABLE; Schema: hdb_catalog; Owner: postgres
--

CREATE TABLE hdb_catalog.hdb_action_log (
    id uuid DEFAULT hdb_catalog.gen_hasura_uuid() NOT NULL,
    action_name text,
    input_payload jsonb NOT NULL,
    request_headers jsonb NOT NULL,
    session_variables jsonb NOT NULL,
    response_payload jsonb,
    errors jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    response_received_at timestamp with time zone,
    status text NOT NULL,
    CONSTRAINT hdb_action_log_status_check CHECK ((status = ANY (ARRAY['created'::text, 'processing'::text, 'completed'::text, 'error'::text])))
);


ALTER TABLE hdb_catalog.hdb_action_log OWNER TO postgres;

--
-- Name: hdb_cron_event_invocation_logs; Type: TABLE; Schema: hdb_catalog; Owner: postgres
--

CREATE TABLE hdb_catalog.hdb_cron_event_invocation_logs (
    id text DEFAULT hdb_catalog.gen_hasura_uuid() NOT NULL,
    event_id text,
    status integer,
    request json,
    response json,
    created_at timestamp with time zone DEFAULT now()
);


ALTER TABLE hdb_catalog.hdb_cron_event_invocation_logs OWNER TO postgres;

--
-- Name: hdb_cron_events; Type: TABLE; Schema: hdb_catalog; Owner: postgres
--

CREATE TABLE hdb_catalog.hdb_cron_events (
    id text DEFAULT hdb_catalog.gen_hasura_uuid() NOT NULL,
    trigger_name text NOT NULL,
    scheduled_time timestamp with time zone NOT NULL,
    status text DEFAULT 'scheduled'::text NOT NULL,
    tries integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    next_retry_at timestamp with time zone,
    CONSTRAINT valid_status CHECK ((status = ANY (ARRAY['scheduled'::text, 'locked'::text, 'delivered'::text, 'error'::text, 'dead'::text])))
);


ALTER TABLE hdb_catalog.hdb_cron_events OWNER TO postgres;

--
-- Name: hdb_metadata; Type: TABLE; Schema: hdb_catalog; Owner: postgres
--

CREATE TABLE hdb_catalog.hdb_metadata (
    id integer NOT NULL,
    metadata json NOT NULL,
    resource_version integer DEFAULT 1 NOT NULL
);


ALTER TABLE hdb_catalog.hdb_metadata OWNER TO postgres;

--
-- Name: hdb_scheduled_event_invocation_logs; Type: TABLE; Schema: hdb_catalog; Owner: postgres
--

CREATE TABLE hdb_catalog.hdb_scheduled_event_invocation_logs (
    id text DEFAULT hdb_catalog.gen_hasura_uuid() NOT NULL,
    event_id text,
    status integer,
    request json,
    response json,
    created_at timestamp with time zone DEFAULT now()
);


ALTER TABLE hdb_catalog.hdb_scheduled_event_invocation_logs OWNER TO postgres;

--
-- Name: hdb_scheduled_events; Type: TABLE; Schema: hdb_catalog; Owner: postgres
--

CREATE TABLE hdb_catalog.hdb_scheduled_events (
    id text DEFAULT hdb_catalog.gen_hasura_uuid() NOT NULL,
    webhook_conf json NOT NULL,
    scheduled_time timestamp with time zone NOT NULL,
    retry_conf json,
    payload json,
    header_conf json,
    status text DEFAULT 'scheduled'::text NOT NULL,
    tries integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    next_retry_at timestamp with time zone,
    comment text,
    CONSTRAINT valid_status CHECK ((status = ANY (ARRAY['scheduled'::text, 'locked'::text, 'delivered'::text, 'error'::text, 'dead'::text])))
);


ALTER TABLE hdb_catalog.hdb_scheduled_events OWNER TO postgres;

--
-- Name: hdb_schema_notifications; Type: TABLE; Schema: hdb_catalog; Owner: postgres
--

CREATE TABLE hdb_catalog.hdb_schema_notifications (
    id integer NOT NULL,
    notification json NOT NULL,
    resource_version integer DEFAULT 1 NOT NULL,
    instance_id uuid NOT NULL,
    updated_at timestamp with time zone DEFAULT now(),
    CONSTRAINT hdb_schema_notifications_id_check CHECK ((id = 1))
);


ALTER TABLE hdb_catalog.hdb_schema_notifications OWNER TO postgres;

--
-- Name: hdb_version; Type: TABLE; Schema: hdb_catalog; Owner: postgres
--

CREATE TABLE hdb_catalog.hdb_version (
    hasura_uuid uuid DEFAULT hdb_catalog.gen_hasura_uuid() NOT NULL,
    version text NOT NULL,
    upgraded_on timestamp with time zone NOT NULL,
    cli_state jsonb DEFAULT '{}'::jsonb NOT NULL,
    console_state jsonb DEFAULT '{}'::jsonb NOT NULL,
    ee_client_id text,
    ee_client_secret text
);


ALTER TABLE hdb_catalog.hdb_version OWNER TO postgres;

--
-- Name: _sqlx_migrations; Type: TABLE; Schema: public; Owner: nikhil
--

CREATE TABLE public._sqlx_migrations (
    version bigint NOT NULL,
    description text NOT NULL,
    installed_on timestamp with time zone DEFAULT now() NOT NULL,
    success boolean NOT NULL,
    checksum bytea NOT NULL,
    execution_time bigint NOT NULL
);


ALTER TABLE public._sqlx_migrations OWNER TO nikhil;

--
-- Name: active_storage_attachments; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.active_storage_attachments (
    id bigint NOT NULL,
    name character varying NOT NULL,
    record_type character varying NOT NULL,
    record_id bigint NOT NULL,
    blob_id bigint NOT NULL,
    created_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.active_storage_attachments OWNER TO postgres;

--
-- Name: active_storage_attachments_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.active_storage_attachments_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.active_storage_attachments_id_seq OWNER TO postgres;

--
-- Name: active_storage_attachments_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.active_storage_attachments_id_seq OWNED BY public.active_storage_attachments.id;


--
-- Name: active_storage_blobs; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.active_storage_blobs (
    id bigint NOT NULL,
    key character varying NOT NULL,
    filename character varying NOT NULL,
    content_type character varying,
    metadata text,
    service_name character varying NOT NULL,
    byte_size bigint NOT NULL,
    checksum character varying,
    created_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.active_storage_blobs OWNER TO postgres;

--
-- Name: active_storage_blobs_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.active_storage_blobs_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.active_storage_blobs_id_seq OWNER TO postgres;

--
-- Name: active_storage_blobs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.active_storage_blobs_id_seq OWNED BY public.active_storage_blobs.id;


--
-- Name: active_storage_variant_records; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.active_storage_variant_records (
    id bigint NOT NULL,
    blob_id bigint NOT NULL,
    variation_digest character varying NOT NULL
);


ALTER TABLE public.active_storage_variant_records OWNER TO postgres;

--
-- Name: active_storage_variant_records_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.active_storage_variant_records_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.active_storage_variant_records_id_seq OWNER TO postgres;

--
-- Name: active_storage_variant_records_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.active_storage_variant_records_id_seq OWNED BY public.active_storage_variant_records.id;


--
-- Name: activity_logs; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.activity_logs (
    id bigint NOT NULL,
    loggable_id integer,
    loggable_type character varying,
    recorder_id integer,
    recorder_type character varying,
    remarks text,
    request_parameter text,
    event_type character varying,
    event_data jsonb,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    relatable_id integer,
    relatable_type character varying
);


ALTER TABLE public.activity_logs OWNER TO postgres;

--
-- Name: activity_logs_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.activity_logs_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.activity_logs_id_seq OWNER TO postgres;

--
-- Name: activity_logs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.activity_logs_id_seq OWNED BY public.activity_logs.id;


--
-- Name: admin_users; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.admin_users (
    id bigint NOT NULL,
    email character varying DEFAULT ''::character varying NOT NULL,
    encrypted_password character varying DEFAULT ''::character varying NOT NULL,
    is_active boolean,
    reset_password_token character varying,
    reset_password_sent_at timestamp(6) without time zone,
    remember_created_at timestamp(6) without time zone,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    invitation_token character varying,
    invitation_created_at timestamp(6) without time zone,
    invitation_sent_at timestamp(6) without time zone,
    invitation_accepted_at timestamp(6) without time zone,
    invitation_limit integer,
    invited_by_type character varying,
    invited_by_id bigint,
    invitations_count integer DEFAULT 0,
    is_super_admin boolean DEFAULT false,
    deleted_at timestamp(6) without time zone,
    otp_secret character varying,
    consumed_timestep integer,
    otp_required_for_login boolean DEFAULT false,
    referral_code character varying,
    is_referral_user boolean DEFAULT false
);


ALTER TABLE public.admin_users OWNER TO postgres;

--
-- Name: admin_users_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.admin_users_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.admin_users_id_seq OWNER TO postgres;

--
-- Name: admin_users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.admin_users_id_seq OWNED BY public.admin_users.id;


--
-- Name: api_request_logs; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.api_request_logs (
    id bigint NOT NULL,
    request_type character varying,
    request_params text,
    response text,
    provider character varying,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    log_id character varying
);


ALTER TABLE public.api_request_logs OWNER TO postgres;

--
-- Name: api_request_logs_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.api_request_logs_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.api_request_logs_id_seq OWNER TO postgres;

--
-- Name: api_request_logs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.api_request_logs_id_seq OWNED BY public.api_request_logs.id;


--
-- Name: application_statuses; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.application_statuses (
    id bigint NOT NULL,
    name character varying,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    deleted_at timestamp(6) without time zone
);


ALTER TABLE public.application_statuses OWNER TO postgres;

--
-- Name: application_statuses_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.application_statuses_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.application_statuses_id_seq OWNER TO postgres;

--
-- Name: application_statuses_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.application_statuses_id_seq OWNED BY public.application_statuses.id;


--
-- Name: ar_internal_metadata; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.ar_internal_metadata (
    key character varying NOT NULL,
    value character varying,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.ar_internal_metadata OWNER TO postgres;

--
-- Name: bank_statements; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.bank_statements (
    id bigint NOT NULL,
    borrower_id integer,
    entity_id character varying,
    account_holder_name character varying,
    link_id character varying,
    bank_name character varying,
    year character varying,
    identity jsonb,
    transactions jsonb,
    salary jsonb,
    recurring_transactions jsonb,
    lender_transactions jsonb,
    deleted_at timestamp(6) without time zone,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    account_details jsonb,
    date_range jsonb,
    frauds jsonb,
    business_id integer,
    monthly_analysis jsonb,
    eod_balances jsonb,
    top_credits_debits jsonb,
    predictors jsonb,
    status character varying,
    doc_holder_type character varying,
    doc_holder_id integer,
    query_id integer
);


ALTER TABLE public.bank_statements OWNER TO postgres;

--
-- Name: bank_statements_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.bank_statements_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.bank_statements_id_seq OWNER TO postgres;

--
-- Name: bank_statements_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.bank_statements_id_seq OWNED BY public.bank_statements.id;


--
-- Name: banking_histories; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.banking_histories (
    id bigint NOT NULL,
    business_id integer,
    year integer,
    month integer,
    day integer,
    amount double precision,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.banking_histories OWNER TO postgres;

--
-- Name: banking_histories_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.banking_histories_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.banking_histories_id_seq OWNER TO postgres;

--
-- Name: banking_histories_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.banking_histories_id_seq OWNED BY public.banking_histories.id;


--
-- Name: borrower_infos; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.borrower_infos (
    id bigint NOT NULL,
    borrower_id integer,
    first_name character varying,
    last_name character varying,
    full_name character varying,
    date_of_birth timestamp(6) without time zone,
    pan_number character varying,
    category character varying,
    address text,
    gender character varying,
    business_identification_number character varying,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.borrower_infos OWNER TO postgres;

--
-- Name: borrower_infos_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.borrower_infos_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.borrower_infos_id_seq OWNER TO postgres;

--
-- Name: borrower_infos_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.borrower_infos_id_seq OWNED BY public.borrower_infos.id;


--
-- Name: borrower_profiles; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.borrower_profiles (
    id bigint NOT NULL,
    institution_id integer,
    product_type_id integer,
    sanctioned_amount double precision,
    outstanding_amount double precision,
    business_id integer,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    borrow_date date,
    no_borrow boolean DEFAULT false,
    business_meta_version character varying,
    loan_taken_on integer
);


ALTER TABLE public.borrower_profiles OWNER TO postgres;

--
-- Name: borrower_profiles_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.borrower_profiles_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.borrower_profiles_id_seq OWNER TO postgres;

--
-- Name: borrower_profiles_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.borrower_profiles_id_seq OWNED BY public.borrower_profiles.id;


--
-- Name: business_metas; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.business_metas (
    id bigint NOT NULL,
    provider character varying,
    version character varying,
    business_id bigint NOT NULL,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    data_type integer,
    data jsonb
);


ALTER TABLE public.business_metas OWNER TO postgres;

--
-- Name: business_metas_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.business_metas_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.business_metas_id_seq OWNER TO postgres;

--
-- Name: business_metas_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.business_metas_id_seq OWNED BY public.business_metas.id;


--
-- Name: business_types; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.business_types (
    id bigint NOT NULL,
    name character varying,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    identification_label character varying,
    deleted_at timestamp(6) without time zone
);


ALTER TABLE public.business_types OWNER TO postgres;

--
-- Name: business_types_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.business_types_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.business_types_id_seq OWNER TO postgres;

--
-- Name: business_types_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.business_types_id_seq OWNED BY public.business_types.id;


--
-- Name: businesses; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.businesses (
    id bigint NOT NULL,
    business_locations text[] DEFAULT '{}'::text[],
    business_model integer,
    about_business text,
    has_outstanding_borrowings boolean,
    borrower_id integer NOT NULL,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    name character varying,
    contact_number character varying,
    business_identification_number character varying,
    avail_whatsapp_notification boolean,
    business_type_id bigint NOT NULL,
    contact_name character varying,
    employee_profile_id bigint,
    operating_locations text[] DEFAULT '{}'::text[],
    last_base_updated timestamp(6) without time zone,
    last_details_updated timestamp(6) without time zone,
    last_fin_year_end date,
    efiling_status character varying,
    next_cin character varying,
    last_filing_date date,
    industry_type_id bigint,
    comprehensive_update_status character varying,
    base_referance_id character varying,
    comprehensive_referance_id character varying
);


ALTER TABLE public.businesses OWNER TO postgres;

--
-- Name: businesses_employee_profiles; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.businesses_employee_profiles (
    business_id bigint NOT NULL,
    employee_profile_id bigint NOT NULL
);


ALTER TABLE public.businesses_employee_profiles OWNER TO postgres;

--
-- Name: businesses_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.businesses_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.businesses_id_seq OWNER TO postgres;

--
-- Name: businesses_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.businesses_id_seq OWNED BY public.businesses.id;


--
-- Name: carts; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.carts (
    id bigint NOT NULL,
    borrower_id integer,
    product_details_id integer,
    query_id integer,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.carts OWNER TO postgres;

--
-- Name: carts_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.carts_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.carts_id_seq OWNER TO postgres;

--
-- Name: carts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.carts_id_seq OWNED BY public.carts.id;


--
-- Name: collaterals; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.collaterals (
    id bigint NOT NULL,
    property_type_id integer,
    possession_type_id integer,
    market_value double precision,
    registry_value double precision,
    address text,
    business_id integer,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    possession_date date,
    no_possession boolean DEFAULT false
);


ALTER TABLE public.collaterals OWNER TO postgres;

--
-- Name: collaterals_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.collaterals_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.collaterals_id_seq OWNER TO postgres;

--
-- Name: collaterals_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.collaterals_id_seq OWNED BY public.collaterals.id;


--
-- Name: comments; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.comments (
    id bigint NOT NULL,
    loan_application_id bigint,
    message text,
    commentable_id bigint,
    commentable_type character varying,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    is_read boolean DEFAULT false
);


ALTER TABLE public.comments OWNER TO postgres;

--
-- Name: comments_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.comments_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.comments_id_seq OWNER TO postgres;

--
-- Name: comments_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.comments_id_seq OWNED BY public.comments.id;


--
-- Name: credit_reports; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.credit_reports (
    id bigint NOT NULL,
    borrower_id integer,
    pan_no character varying,
    first_name character varying,
    last_name character varying,
    date_of_birth timestamp(6) without time zone,
    gender integer,
    mobile_no character varying,
    email_id character varying,
    address character varying,
    city character varying,
    state integer,
    pincode character varying,
    stage_one_hit_id character varying,
    stage_two_hit_id character varying,
    html_report text,
    xml_report text,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    response_message text,
    score integer,
    data jsonb
);


ALTER TABLE public.credit_reports OWNER TO postgres;

--
-- Name: credit_reports_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.credit_reports_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.credit_reports_id_seq OWNER TO postgres;

--
-- Name: credit_reports_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.credit_reports_id_seq OWNED BY public.credit_reports.id;


--
-- Name: designations; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.designations (
    id bigint NOT NULL,
    name character varying,
    slug character varying,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.designations OWNER TO postgres;

--
-- Name: designations_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.designations_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.designations_id_seq OWNER TO postgres;

--
-- Name: designations_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.designations_id_seq OWNED BY public.designations.id;


--
-- Name: directors; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.directors (
    id bigint NOT NULL,
    name character varying,
    pan character varying,
    din character varying,
    designation character varying,
    date_of_joining date,
    business_meta_version character varying,
    business_id bigint NOT NULL,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    gender character varying,
    date_of_birth character varying,
    age integer,
    father_name character varying,
    address jsonb
);


ALTER TABLE public.directors OWNER TO postgres;

--
-- Name: directors_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.directors_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.directors_id_seq OWNER TO postgres;

--
-- Name: directors_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.directors_id_seq OWNED BY public.directors.id;


--
-- Name: disbursements; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.disbursements (
    id bigint NOT NULL,
    amount double precision,
    product_id integer,
    institution_id integer,
    employee_profile_id integer,
    loan_application_id integer,
    loan_match_id integer,
    disbursement_at timestamp(6) without time zone,
    disbursement_comments text,
    status_id integer,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    send_email boolean DEFAULT false,
    to_email_addresses text,
    account_holder_name character varying,
    rate_of_interest double precision,
    processing_fees double precision,
    emi_date character varying,
    emi_amount double precision,
    tenure integer,
    bank_account_details text,
    foreclosure_charges text
);


ALTER TABLE public.disbursements OWNER TO postgres;

--
-- Name: disbursements_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.disbursements_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.disbursements_id_seq OWNER TO postgres;

--
-- Name: disbursements_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.disbursements_id_seq OWNED BY public.disbursements.id;


--
-- Name: document_types; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.document_types (
    id bigint NOT NULL,
    name character varying,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    deleted_at timestamp(6) without time zone
);


ALTER TABLE public.document_types OWNER TO postgres;

--
-- Name: document_types_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.document_types_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.document_types_id_seq OWNER TO postgres;

--
-- Name: document_types_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.document_types_id_seq OWNED BY public.document_types.id;


--
-- Name: documents; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.documents (
    id bigint NOT NULL,
    document_type_id integer,
    business_id integer,
    name text,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.documents OWNER TO postgres;

--
-- Name: documents_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.documents_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.documents_id_seq OWNER TO postgres;

--
-- Name: documents_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.documents_id_seq OWNED BY public.documents.id;


--
-- Name: employee_profiles; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.employee_profiles (
    id bigint NOT NULL,
    name character varying,
    contact_number character varying,
    is_active boolean,
    admin_user_id bigint NOT NULL,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    location_id bigint NOT NULL
);


ALTER TABLE public.employee_profiles OWNER TO postgres;

--
-- Name: employee_profiles_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.employee_profiles_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.employee_profiles_id_seq OWNER TO postgres;

--
-- Name: employee_profiles_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.employee_profiles_id_seq OWNED BY public.employee_profiles.id;


--
-- Name: employee_profiles_loan_applications; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.employee_profiles_loan_applications (
    loan_application_id bigint NOT NULL,
    employee_profile_id bigint NOT NULL
);


ALTER TABLE public.employee_profiles_loan_applications OWNER TO postgres;

--
-- Name: finance_records; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.finance_records (
    id bigint NOT NULL,
    borrower_id integer,
    business_id integer,
    avg_sales double precision,
    avg_depreciation double precision,
    avg_interest_on_loan double precision,
    avg_other_income_interest double precision,
    avg_cost_of_goods double precision,
    avg_net_profit_before_tax double precision,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.finance_records OWNER TO postgres;

--
-- Name: finance_records_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.finance_records_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.finance_records_id_seq OWNER TO postgres;

--
-- Name: finance_records_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.finance_records_id_seq OWNED BY public.finance_records.id;


--
-- Name: financial_ratios; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.financial_ratios (
    id bigint NOT NULL,
    year character varying,
    revenue_growth double precision,
    ebitda_margin double precision,
    net_margin double precision,
    return_on_equity double precision,
    debt_by_equity double precision,
    sales_by_net_fixed_assets double precision,
    cash_conversion_cycle double precision,
    business_meta_version character varying,
    business_id bigint NOT NULL,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.financial_ratios OWNER TO postgres;

--
-- Name: financial_ratios_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.financial_ratios_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.financial_ratios_id_seq OWNER TO postgres;

--
-- Name: financial_ratios_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.financial_ratios_id_seq OWNED BY public.financial_ratios.id;


--
-- Name: financial_summaries; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.financial_summaries (
    id bigint NOT NULL,
    year character varying,
    sales double precision,
    depreciation double precision,
    finance_costs double precision,
    business_meta_version character varying,
    business_id bigint NOT NULL,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    total_assets double precision,
    operating_profit double precision,
    income_tax double precision,
    profit_for_the_period double precision,
    total_equity double precision,
    total_liabilities double precision,
    other_income double precision DEFAULT 0.0,
    interest double precision DEFAULT 0.0,
    cost_of_goods double precision DEFAULT 0.0,
    rental_income double precision DEFAULT 0.0,
    net_profit_before_tax double precision DEFAULT 0.0,
    profit_after_tax double precision,
    profit_before_interest_and_tax double precision,
    itr_report_id integer
);


ALTER TABLE public.financial_summaries OWNER TO postgres;

--
-- Name: financial_summaries_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.financial_summaries_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.financial_summaries_id_seq OWNER TO postgres;

--
-- Name: financial_summaries_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.financial_summaries_id_seq OWNED BY public.financial_summaries.id;


--
-- Name: gst_basic_details; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.gst_basic_details (
    id bigint NOT NULL,
    state character varying,
    gstin character varying,
    status character varying,
    "gstinFilingDetails" jsonb,
    "gstinDetails" jsonb,
    borrower_id integer,
    business_id integer,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.gst_basic_details OWNER TO postgres;

--
-- Name: gst_basic_details_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.gst_basic_details_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.gst_basic_details_id_seq OWNER TO postgres;

--
-- Name: gst_basic_details_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.gst_basic_details_id_seq OWNED BY public.gst_basic_details.id;


--
-- Name: gst_basic_details_queries; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.gst_basic_details_queries (
    query_id bigint NOT NULL,
    gst_basic_detail_id bigint NOT NULL
);


ALTER TABLE public.gst_basic_details_queries OWNER TO postgres;

--
-- Name: gst_reports; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.gst_reports (
    id bigint NOT NULL,
    borrower_id integer,
    business_id integer,
    reference_id character varying,
    data jsonb,
    gstin character varying,
    from_date character varying,
    to_date character varying,
    status character varying,
    report_url character varying,
    user_name character varying,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    deleted_at timestamp(6) without time zone,
    api_type character varying,
    email character varying,
    gst_basic_detail_id integer,
    loc_state character varying,
    query_id integer
);


ALTER TABLE public.gst_reports OWNER TO postgres;

--
-- Name: gst_reports_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.gst_reports_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.gst_reports_id_seq OWNER TO postgres;

--
-- Name: gst_reports_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.gst_reports_id_seq OWNED BY public.gst_reports.id;


--
-- Name: gst_returns; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.gst_returns (
    id bigint NOT NULL,
    year integer,
    month integer,
    amount double precision,
    business_id bigint NOT NULL,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.gst_returns OWNER TO postgres;

--
-- Name: gst_returns_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.gst_returns_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.gst_returns_id_seq OWNER TO postgres;

--
-- Name: gst_returns_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.gst_returns_id_seq OWNED BY public.gst_returns.id;


--
-- Name: industry_types; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.industry_types (
    id bigint NOT NULL,
    name character varying,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    margin double precision,
    deleted_at timestamp(6) without time zone
);


ALTER TABLE public.industry_types OWNER TO postgres;

--
-- Name: industry_types_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.industry_types_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.industry_types_id_seq OWNER TO postgres;

--
-- Name: industry_types_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.industry_types_id_seq OWNED BY public.industry_types.id;


--
-- Name: institution_types; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.institution_types (
    id bigint NOT NULL,
    name character varying,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    deleted_at timestamp(6) without time zone
);


ALTER TABLE public.institution_types OWNER TO postgres;

--
-- Name: institution_types_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.institution_types_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.institution_types_id_seq OWNER TO postgres;

--
-- Name: institution_types_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.institution_types_id_seq OWNED BY public.institution_types.id;


--
-- Name: institutions; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.institutions (
    id bigint NOT NULL,
    name character varying,
    website character varying,
    is_active boolean,
    institution_type_id bigint NOT NULL,
    location_id bigint NOT NULL,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    deleted_at timestamp(6) without time zone
);


ALTER TABLE public.institutions OWNER TO postgres;

--
-- Name: institutions_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.institutions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.institutions_id_seq OWNER TO postgres;

--
-- Name: institutions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.institutions_id_seq OWNED BY public.institutions.id;


--
-- Name: instrument_relationships; Type: TABLE; Schema: public; Owner: nikhil
--

CREATE TABLE public.instrument_relationships (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    source_instrument_id uuid NOT NULL,
    target_instrument_id uuid NOT NULL,
    relationship_type character varying(50) NOT NULL,
    metadata jsonb DEFAULT '{}'::jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.instrument_relationships OWNER TO nikhil;

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
-- Name: itr_reports; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.itr_reports (
    id bigint NOT NULL,
    borrower_id integer,
    business_id integer,
    reference_id character varying,
    data jsonb,
    pan character varying,
    from_date character varying,
    to_date character varying,
    status character varying,
    report_url character varying,
    user_name character varying,
    api_type character varying,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    deleted_at timestamp(6) without time zone,
    doc_holder_type character varying,
    doc_holder_id integer
);


ALTER TABLE public.itr_reports OWNER TO postgres;

--
-- Name: itr_reports_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.itr_reports_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.itr_reports_id_seq OWNER TO postgres;

--
-- Name: itr_reports_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.itr_reports_id_seq OWNED BY public.itr_reports.id;


--
-- Name: itr_reports_queries; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.itr_reports_queries (
    query_id bigint NOT NULL,
    itr_report_id bigint NOT NULL
);


ALTER TABLE public.itr_reports_queries OWNER TO postgres;

--
-- Name: jwt_denylist; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.jwt_denylist (
    id bigint NOT NULL,
    jti character varying NOT NULL,
    exp timestamp(6) without time zone NOT NULL,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.jwt_denylist OWNER TO postgres;

--
-- Name: jwt_denylist_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.jwt_denylist_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.jwt_denylist_id_seq OWNER TO postgres;

--
-- Name: jwt_denylist_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.jwt_denylist_id_seq OWNED BY public.jwt_denylist.id;


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
    recorded_at timestamp with time zone DEFAULT now() NOT NULL
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
-- Name: leads; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.leads (
    id bigint NOT NULL,
    loan_match_id integer,
    status_id integer,
    business_id integer,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    query_id integer,
    created_by_type character varying,
    created_by_id integer
);


ALTER TABLE public.leads OWNER TO postgres;

--
-- Name: leads_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.leads_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.leads_id_seq OWNER TO postgres;

--
-- Name: leads_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.leads_id_seq OWNED BY public.leads.id;


--
-- Name: line_items; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.line_items (
    id bigint NOT NULL,
    query_id integer,
    borrower_id integer,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    loan_match_id integer,
    added_by_type character varying,
    added_by_id integer
);


ALTER TABLE public.line_items OWNER TO postgres;

--
-- Name: line_items_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.line_items_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.line_items_id_seq OWNER TO postgres;

--
-- Name: line_items_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.line_items_id_seq OWNED BY public.line_items.id;


--
-- Name: loan_applications; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.loan_applications (
    id bigint NOT NULL,
    loan_amount double precision,
    credit_score integer,
    monthly_emi double precision DEFAULT 0.0,
    product_type_id bigint NOT NULL,
    property_type_id bigint,
    business_id bigint NOT NULL,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    industry_type_str character varying DEFAULT ''::character varying,
    property_type_str character varying DEFAULT ''::character varying,
    property_sub_type_str character varying DEFAULT ''::character varying,
    tenure double precision DEFAULT 0.0,
    sales double precision DEFAULT 0.0,
    cost_of_goods double precision DEFAULT 0.0,
    depreciation double precision DEFAULT 0.0,
    interest_on_loan double precision DEFAULT 0.0,
    director_partner_remuneration double precision DEFAULT 0.0,
    rental double precision DEFAULT 0.0,
    other_income_interest double precision DEFAULT 0.0,
    obligation double precision DEFAULT 0.0,
    net_profit_before_tax double precision DEFAULT 0.0,
    last_12_mnths_turnover double precision DEFAULT 0.0,
    cash_profit double precision DEFAULT 0.0,
    gross_profit double precision DEFAULT 0.0,
    property_value double precision DEFAULT 0.0,
    se_cash_profit double precision DEFAULT 0.0,
    se_gross_profit double precision DEFAULT 0.0,
    se_gross_margin double precision DEFAULT 0.0,
    avg_monthly_bala double precision DEFAULT 0.0,
    loan_type_id integer,
    applicant_name character varying,
    co_applicant_name character varying,
    income double precision,
    property_market_value double precision,
    property_registry_value double precision,
    borrower_profile_id integer,
    existing_loan_institution_id integer,
    loan_amount_taken double precision,
    loan_year integer,
    outstanding_loan_amount double precision,
    loan_property_type_id integer,
    loan_property_value double precision,
    ltv_taken double precision,
    application_status_id integer,
    deleted_at timestamp(6) without time zone,
    status character varying,
    applicant_type integer,
    co_applicant_type integer,
    co_applicant_credit_score integer,
    loan_application_no character varying,
    api_request_log_id character varying
);


ALTER TABLE public.loan_applications OWNER TO postgres;

--
-- Name: loan_applications_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.loan_applications_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.loan_applications_id_seq OWNER TO postgres;

--
-- Name: loan_applications_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.loan_applications_id_seq OWNED BY public.loan_applications.id;


--
-- Name: loan_matches; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.loan_matches (
    id bigint NOT NULL,
    loan_application_id integer,
    product_id integer,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    expected_loan_amount double precision DEFAULT 0.0,
    is_lead boolean DEFAULT false,
    aasm_state character varying,
    admin_user_id integer,
    lender_id integer,
    is_published boolean,
    published_by_id integer,
    published_by_type character varying,
    published_at timestamp(6) without time zone,
    deleted_at timestamp(6) without time zone,
    interested_on timestamp(6) without time zone,
    status_id integer,
    product_detail_id integer,
    remark text,
    net_profit_before_tax double precision DEFAULT 0.0,
    depreciation double precision DEFAULT 0.0,
    interest_on_loan double precision DEFAULT 0.0,
    cash_profit double precision DEFAULT 0.0,
    foir double precision DEFAULT 0.0,
    obligation double precision DEFAULT 0.0,
    rate_of_interest double precision DEFAULT 0.0,
    tenure double precision DEFAULT 0.0,
    property_market_value double precision DEFAULT 0.0,
    ltv double precision DEFAULT 0.0,
    rental_income double precision DEFAULT 0.0,
    other_income_interest double precision DEFAULT 0.0,
    director_remuneration double precision DEFAULT 0.0,
    avg_operating_profit double precision DEFAULT 0.0,
    sales double precision DEFAULT 0.0,
    cost_of_goods double precision DEFAULT 0.0,
    multiplier double precision DEFAULT 0.0,
    tenure_of_banking_in_month integer,
    avg_balance_date character varying,
    cart_status character varying,
    query_type character varying,
    query_id integer,
    total_gst_taxable_value double precision,
    industry_margin double precision
);


ALTER TABLE public.loan_matches OWNER TO postgres;

--
-- Name: loan_matches_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.loan_matches_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.loan_matches_id_seq OWNER TO postgres;

--
-- Name: loan_matches_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.loan_matches_id_seq OWNED BY public.loan_matches.id;


--
-- Name: loan_types; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.loan_types (
    id bigint NOT NULL,
    name character varying,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.loan_types OWNER TO postgres;

--
-- Name: loan_types_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.loan_types_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.loan_types_id_seq OWNER TO postgres;

--
-- Name: loan_types_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.loan_types_id_seq OWNED BY public.loan_types.id;


--
-- Name: locations; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.locations (
    id bigint NOT NULL,
    name character varying,
    latitude double precision,
    longitude double precision,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    deleted_at timestamp(6) without time zone
);


ALTER TABLE public.locations OWNER TO postgres;

--
-- Name: locations_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.locations_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.locations_id_seq OWNER TO postgres;

--
-- Name: locations_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.locations_id_seq OWNED BY public.locations.id;


--
-- Name: meetings; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.meetings (
    id bigint NOT NULL,
    title text,
    description text,
    start_time timestamp(6) without time zone,
    end_time timestamp(6) without time zone,
    link character varying,
    loan_match_id bigint NOT NULL,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.meetings OWNER TO postgres;

--
-- Name: meetings_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.meetings_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.meetings_id_seq OWNER TO postgres;

--
-- Name: meetings_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.meetings_id_seq OWNED BY public.meetings.id;


--
-- Name: observations; Type: TABLE; Schema: public; Owner: nikhil
--

CREATE TABLE public.observations (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    observation_type character varying(50) NOT NULL,
    observed_at timestamp with time zone NOT NULL,
    effective_from timestamp with time zone,
    effective_to timestamp with time zone,
    recorded_at timestamp with time zone DEFAULT now() NOT NULL,
    instrument_id uuid,
    raw_payload jsonb DEFAULT '{}'::jsonb,
    normalized_payload jsonb DEFAULT '{}'::jsonb,
    confidence_score double precision DEFAULT 1.0 NOT NULL,
    freshness_at double precision DEFAULT 0.0 NOT NULL,
    quality_score double precision DEFAULT 1.0 NOT NULL,
    source_name character varying(255) NOT NULL,
    coverage character varying(50) DEFAULT 'Complete'::character varying NOT NULL,
    consistency_score double precision,
    provenance_hash character varying(64) NOT NULL,
    schema_version integer DEFAULT 1 NOT NULL
);


ALTER TABLE public.observations OWNER TO nikhil;

--
-- Name: product_details; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.product_details (
    id bigint NOT NULL,
    product_id integer,
    product_sub_type_id integer,
    property_type_id integer,
    program_id integer,
    min_loan_amount double precision,
    max_loan_amount double precision,
    min_tenure integer,
    max_tenure integer,
    min_credit_score integer,
    bto double precision,
    gearing_ratio double precision,
    tol double precision,
    dscr double precision,
    ltv double precision,
    foir double precision,
    rate_of_interest double precision,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    avg_balance_date character varying,
    multiplier double precision DEFAULT 0.0,
    tenure_of_banking_in_month integer,
    industry_margin double precision,
    gst_obligation_period integer,
    gst_report_months integer
);


ALTER TABLE public.product_details OWNER TO postgres;

--
-- Name: product_details_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.product_details_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.product_details_id_seq OWNER TO postgres;

--
-- Name: product_details_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.product_details_id_seq OWNED BY public.product_details.id;


--
-- Name: product_details_queries; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.product_details_queries (
    query_id bigint NOT NULL,
    product_detail_id bigint NOT NULL
);


ALTER TABLE public.product_details_queries OWNER TO postgres;

--
-- Name: product_types; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.product_types (
    id bigint NOT NULL,
    name character varying,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    parent_id integer,
    extra_fields text[] DEFAULT '{}'::text[],
    deleted_at timestamp(6) without time zone,
    tenure_in_month integer,
    category character varying
);


ALTER TABLE public.product_types OWNER TO postgres;

--
-- Name: product_types_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.product_types_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.product_types_id_seq OWNER TO postgres;

--
-- Name: product_types_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.product_types_id_seq OWNED BY public.product_types.id;


--
-- Name: product_types_property_types; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.product_types_property_types (
    product_type_id bigint,
    property_type_id bigint,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.product_types_property_types OWNER TO postgres;

--
-- Name: products; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.products (
    id bigint NOT NULL,
    product_type_id integer,
    loan_types text[] DEFAULT '{}'::text[],
    institution_id integer,
    lender_id integer,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    deleted_at timestamp(6) without time zone
);


ALTER TABLE public.products OWNER TO postgres;

--
-- Name: products_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.products_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.products_id_seq OWNER TO postgres;

--
-- Name: products_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.products_id_seq OWNED BY public.products.id;


--
-- Name: profile_views; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.profile_views (
    id bigint NOT NULL,
    borrower_id integer,
    lender_id integer,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.profile_views OWNER TO postgres;

--
-- Name: profile_views_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.profile_views_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.profile_views_id_seq OWNER TO postgres;

--
-- Name: profile_views_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.profile_views_id_seq OWNED BY public.profile_views.id;


--
-- Name: profiles; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.profiles (
    id bigint NOT NULL,
    name character varying,
    phone_no bigint,
    title character varying,
    location character varying,
    level character varying,
    rating double precision,
    description text,
    specialization character varying,
    service_locations text[] DEFAULT '{}'::text[],
    availability character varying,
    is_featured boolean,
    lender_id bigint NOT NULL,
    institution_id bigint NOT NULL,
    relationship_manager_id integer,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    employee_profile_id bigint,
    deleted_at timestamp(6) without time zone
);


ALTER TABLE public.profiles OWNER TO postgres;

--
-- Name: profiles_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.profiles_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.profiles_id_seq OWNER TO postgres;

--
-- Name: profiles_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.profiles_id_seq OWNED BY public.profiles.id;


--
-- Name: programs; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.programs (
    id bigint NOT NULL,
    name character varying,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    deleted_at timestamp(6) without time zone
);


ALTER TABLE public.programs OWNER TO postgres;

--
-- Name: programs_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.programs_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.programs_id_seq OWNER TO postgres;

--
-- Name: programs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.programs_id_seq OWNED BY public.programs.id;


--
-- Name: promoters; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.promoters (
    id bigint NOT NULL,
    name character varying,
    pan character varying,
    date_of_appointment date,
    business_meta_version character varying,
    business_id bigint NOT NULL,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.promoters OWNER TO postgres;

--
-- Name: promoters_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.promoters_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.promoters_id_seq OWNER TO postgres;

--
-- Name: promoters_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.promoters_id_seq OWNED BY public.promoters.id;


--
-- Name: property_types; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.property_types (
    id bigint NOT NULL,
    name character varying,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    parent_id bigint
);


ALTER TABLE public.property_types OWNER TO postgres;

--
-- Name: property_types_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.property_types_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.property_types_id_seq OWNER TO postgres;

--
-- Name: property_types_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.property_types_id_seq OWNED BY public.property_types.id;


--
-- Name: queries; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.queries (
    id bigint NOT NULL,
    loan_amount character varying,
    product_type_id integer,
    max_tenure_in_month integer,
    min_tenure_in_month integer,
    credit_score integer,
    loan_type character varying,
    loan_amount_range character varying,
    borrower_id integer,
    business_id integer,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    status_id integer,
    net_profit_before_tax double precision DEFAULT 0.0,
    depreciation double precision DEFAULT 0.0,
    interest_on_loan double precision DEFAULT 0.0,
    obligation double precision DEFAULT 0.0,
    property_market_value double precision DEFAULT 0.0,
    cash_profit double precision DEFAULT 0.0,
    sales double precision DEFAULT 0.0,
    cost_of_goods double precision DEFAULT 0.0,
    gross_profit double precision
);


ALTER TABLE public.queries OWNER TO postgres;

--
-- Name: queries_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.queries_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.queries_id_seq OWNER TO postgres;

--
-- Name: queries_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.queries_id_seq OWNED BY public.queries.id;


--
-- Name: referral_codes; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.referral_codes (
    id bigint NOT NULL,
    code character varying,
    referrer_name character varying,
    contact_number character varying,
    referrer_type character varying,
    referrer_id bigint,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.referral_codes OWNER TO postgres;

--
-- Name: referral_codes_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.referral_codes_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.referral_codes_id_seq OWNER TO postgres;

--
-- Name: referral_codes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.referral_codes_id_seq OWNED BY public.referral_codes.id;


--
-- Name: referral_types; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.referral_types (
    id bigint NOT NULL,
    name character varying,
    label character varying,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.referral_types OWNER TO postgres;

--
-- Name: referral_types_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.referral_types_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.referral_types_id_seq OWNER TO postgres;

--
-- Name: referral_types_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.referral_types_id_seq OWNED BY public.referral_types.id;


--
-- Name: referrer_branch_cities; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.referrer_branch_cities (
    id bigint NOT NULL,
    zone character varying,
    location character varying,
    referrer_id integer,
    branch_head_id integer,
    zone_head_id integer,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.referrer_branch_cities OWNER TO postgres;

--
-- Name: referrer_branch_cities_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.referrer_branch_cities_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.referrer_branch_cities_id_seq OWNER TO postgres;

--
-- Name: referrer_branch_cities_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.referrer_branch_cities_id_seq OWNED BY public.referrer_branch_cities.id;


--
-- Name: research_sessions; Type: TABLE; Schema: public; Owner: nikhil
--

CREATE TABLE public.research_sessions (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    title character varying(255) NOT NULL,
    description text,
    status character varying(50) DEFAULT 'Open'::character varying NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    closed_at timestamp with time zone
);


ALTER TABLE public.research_sessions OWNER TO nikhil;

--
-- Name: roles; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.roles (
    id bigint NOT NULL,
    admin_user_id bigint NOT NULL,
    designation_id bigint NOT NULL,
    manager_emp_id integer,
    created_by integer,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.roles OWNER TO postgres;

--
-- Name: roles_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.roles_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.roles_id_seq OWNER TO postgres;

--
-- Name: roles_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.roles_id_seq OWNED BY public.roles.id;


--
-- Name: schema_migrations; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.schema_migrations (
    version character varying NOT NULL
);


ALTER TABLE public.schema_migrations OWNER TO postgres;

--
-- Name: shareholdings; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.shareholdings (
    id bigint NOT NULL,
    business_id integer,
    shareholder_name character varying,
    holding_percentage double precision,
    din_pan character varying,
    designation character varying,
    date_of_cessation character varying,
    no_of_shares character varying,
    business_meta_version character varying,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL
);


ALTER TABLE public.shareholdings OWNER TO postgres;

--
-- Name: shareholdings_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.shareholdings_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.shareholdings_id_seq OWNER TO postgres;

--
-- Name: shareholdings_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.shareholdings_id_seq OWNED BY public.shareholdings.id;


--
-- Name: signup_requests; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.signup_requests (
    id bigint NOT NULL,
    email character varying,
    user_type character varying,
    mobile_number character varying,
    verification_otp integer,
    is_otp_verified boolean,
    status integer,
    referral_code character varying,
    created_by character varying,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    user_id integer
);


ALTER TABLE public.signup_requests OWNER TO postgres;

--
-- Name: signup_requests_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.signup_requests_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.signup_requests_id_seq OWNER TO postgres;

--
-- Name: signup_requests_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.signup_requests_id_seq OWNED BY public.signup_requests.id;


--
-- Name: tata_apis; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.tata_apis (
    id bigint NOT NULL,
    lead_id character varying DEFAULT ''::character varying,
    webtop_id character varying DEFAULT ''::character varying,
    opportunity_id character varying DEFAULT ''::character varying,
    borrower_id integer NOT NULL,
    query_id integer,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    application_number character varying
);


ALTER TABLE public.tata_apis OWNER TO postgres;

--
-- Name: tata_apis_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.tata_apis_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.tata_apis_id_seq OWNER TO postgres;

--
-- Name: tata_apis_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.tata_apis_id_seq OWNED BY public.tata_apis.id;


--
-- Name: users; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.users (
    id bigint NOT NULL,
    email character varying DEFAULT ''::character varying NOT NULL,
    encrypted_password character varying DEFAULT ''::character varying NOT NULL,
    reset_password_token character varying,
    reset_password_sent_at timestamp(6) without time zone,
    remember_created_at timestamp(6) without time zone,
    type character varying,
    created_at timestamp(6) without time zone NOT NULL,
    updated_at timestamp(6) without time zone NOT NULL,
    invitation_token character varying,
    invitation_created_at timestamp(6) without time zone,
    invitation_sent_at timestamp(6) without time zone,
    invitation_accepted_at timestamp(6) without time zone,
    invitation_limit integer,
    invited_by_type character varying,
    invited_by_id bigint,
    invitations_count integer DEFAULT 0,
    confirmation_token character varying,
    confirmed_at timestamp(6) without time zone,
    confirmation_sent_at timestamp(6) without time zone,
    unconfirmed_email character varying,
    sign_in_count integer DEFAULT 0 NOT NULL,
    current_sign_in_at timestamp(6) without time zone,
    last_sign_in_at timestamp(6) without time zone,
    current_sign_in_ip character varying,
    last_sign_in_ip character varying,
    deleted_at timestamp(6) without time zone,
    mobile_number character varying,
    verification_otp integer,
    is_otp_verified boolean DEFAULT false,
    login_otp integer,
    is_login_otp_verified boolean DEFAULT false,
    created_by character varying,
    referral_code character varying,
    referred_by_id integer,
    referred_by_type character varying,
    borrower_click_count integer DEFAULT 0,
    name character varying,
    is_active boolean DEFAULT true,
    referrer_click_count integer DEFAULT 0
);


ALTER TABLE public.users OWNER TO postgres;

--
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.users_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.users_id_seq OWNER TO postgres;

--
-- Name: users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.users_id_seq OWNED BY public.users.id;


--
-- Name: versions; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.versions (
    id bigint NOT NULL,
    item_type character varying NOT NULL,
    item_id bigint NOT NULL,
    event character varying NOT NULL,
    whodunnit character varying,
    object text,
    created_at timestamp(6) without time zone
);


ALTER TABLE public.versions OWNER TO postgres;

--
-- Name: versions_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.versions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.versions_id_seq OWNER TO postgres;

--
-- Name: versions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.versions_id_seq OWNED BY public.versions.id;


--
-- Name: active_storage_attachments id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.active_storage_attachments ALTER COLUMN id SET DEFAULT nextval('public.active_storage_attachments_id_seq'::regclass);


--
-- Name: active_storage_blobs id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.active_storage_blobs ALTER COLUMN id SET DEFAULT nextval('public.active_storage_blobs_id_seq'::regclass);


--
-- Name: active_storage_variant_records id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.active_storage_variant_records ALTER COLUMN id SET DEFAULT nextval('public.active_storage_variant_records_id_seq'::regclass);


--
-- Name: activity_logs id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.activity_logs ALTER COLUMN id SET DEFAULT nextval('public.activity_logs_id_seq'::regclass);


--
-- Name: admin_users id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.admin_users ALTER COLUMN id SET DEFAULT nextval('public.admin_users_id_seq'::regclass);


--
-- Name: api_request_logs id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.api_request_logs ALTER COLUMN id SET DEFAULT nextval('public.api_request_logs_id_seq'::regclass);


--
-- Name: application_statuses id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.application_statuses ALTER COLUMN id SET DEFAULT nextval('public.application_statuses_id_seq'::regclass);


--
-- Name: bank_statements id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.bank_statements ALTER COLUMN id SET DEFAULT nextval('public.bank_statements_id_seq'::regclass);


--
-- Name: banking_histories id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.banking_histories ALTER COLUMN id SET DEFAULT nextval('public.banking_histories_id_seq'::regclass);


--
-- Name: borrower_infos id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.borrower_infos ALTER COLUMN id SET DEFAULT nextval('public.borrower_infos_id_seq'::regclass);


--
-- Name: borrower_profiles id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.borrower_profiles ALTER COLUMN id SET DEFAULT nextval('public.borrower_profiles_id_seq'::regclass);


--
-- Name: business_metas id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.business_metas ALTER COLUMN id SET DEFAULT nextval('public.business_metas_id_seq'::regclass);


--
-- Name: business_types id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.business_types ALTER COLUMN id SET DEFAULT nextval('public.business_types_id_seq'::regclass);


--
-- Name: businesses id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.businesses ALTER COLUMN id SET DEFAULT nextval('public.businesses_id_seq'::regclass);


--
-- Name: carts id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.carts ALTER COLUMN id SET DEFAULT nextval('public.carts_id_seq'::regclass);


--
-- Name: collaterals id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.collaterals ALTER COLUMN id SET DEFAULT nextval('public.collaterals_id_seq'::regclass);


--
-- Name: comments id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.comments ALTER COLUMN id SET DEFAULT nextval('public.comments_id_seq'::regclass);


--
-- Name: credit_reports id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.credit_reports ALTER COLUMN id SET DEFAULT nextval('public.credit_reports_id_seq'::regclass);


--
-- Name: designations id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.designations ALTER COLUMN id SET DEFAULT nextval('public.designations_id_seq'::regclass);


--
-- Name: directors id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.directors ALTER COLUMN id SET DEFAULT nextval('public.directors_id_seq'::regclass);


--
-- Name: disbursements id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.disbursements ALTER COLUMN id SET DEFAULT nextval('public.disbursements_id_seq'::regclass);


--
-- Name: document_types id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.document_types ALTER COLUMN id SET DEFAULT nextval('public.document_types_id_seq'::regclass);


--
-- Name: documents id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.documents ALTER COLUMN id SET DEFAULT nextval('public.documents_id_seq'::regclass);


--
-- Name: employee_profiles id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.employee_profiles ALTER COLUMN id SET DEFAULT nextval('public.employee_profiles_id_seq'::regclass);


--
-- Name: finance_records id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.finance_records ALTER COLUMN id SET DEFAULT nextval('public.finance_records_id_seq'::regclass);


--
-- Name: financial_ratios id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.financial_ratios ALTER COLUMN id SET DEFAULT nextval('public.financial_ratios_id_seq'::regclass);


--
-- Name: financial_summaries id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.financial_summaries ALTER COLUMN id SET DEFAULT nextval('public.financial_summaries_id_seq'::regclass);


--
-- Name: gst_basic_details id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.gst_basic_details ALTER COLUMN id SET DEFAULT nextval('public.gst_basic_details_id_seq'::regclass);


--
-- Name: gst_reports id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.gst_reports ALTER COLUMN id SET DEFAULT nextval('public.gst_reports_id_seq'::regclass);


--
-- Name: gst_returns id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.gst_returns ALTER COLUMN id SET DEFAULT nextval('public.gst_returns_id_seq'::regclass);


--
-- Name: industry_types id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.industry_types ALTER COLUMN id SET DEFAULT nextval('public.industry_types_id_seq'::regclass);


--
-- Name: institution_types id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.institution_types ALTER COLUMN id SET DEFAULT nextval('public.institution_types_id_seq'::regclass);


--
-- Name: institutions id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.institutions ALTER COLUMN id SET DEFAULT nextval('public.institutions_id_seq'::regclass);


--
-- Name: itr_reports id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.itr_reports ALTER COLUMN id SET DEFAULT nextval('public.itr_reports_id_seq'::regclass);


--
-- Name: jwt_denylist id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.jwt_denylist ALTER COLUMN id SET DEFAULT nextval('public.jwt_denylist_id_seq'::regclass);


--
-- Name: leads id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.leads ALTER COLUMN id SET DEFAULT nextval('public.leads_id_seq'::regclass);


--
-- Name: line_items id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.line_items ALTER COLUMN id SET DEFAULT nextval('public.line_items_id_seq'::regclass);


--
-- Name: loan_applications id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.loan_applications ALTER COLUMN id SET DEFAULT nextval('public.loan_applications_id_seq'::regclass);


--
-- Name: loan_matches id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.loan_matches ALTER COLUMN id SET DEFAULT nextval('public.loan_matches_id_seq'::regclass);


--
-- Name: loan_types id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.loan_types ALTER COLUMN id SET DEFAULT nextval('public.loan_types_id_seq'::regclass);


--
-- Name: locations id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.locations ALTER COLUMN id SET DEFAULT nextval('public.locations_id_seq'::regclass);


--
-- Name: meetings id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.meetings ALTER COLUMN id SET DEFAULT nextval('public.meetings_id_seq'::regclass);


--
-- Name: product_details id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.product_details ALTER COLUMN id SET DEFAULT nextval('public.product_details_id_seq'::regclass);


--
-- Name: product_types id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.product_types ALTER COLUMN id SET DEFAULT nextval('public.product_types_id_seq'::regclass);


--
-- Name: products id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.products ALTER COLUMN id SET DEFAULT nextval('public.products_id_seq'::regclass);


--
-- Name: profile_views id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.profile_views ALTER COLUMN id SET DEFAULT nextval('public.profile_views_id_seq'::regclass);


--
-- Name: profiles id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.profiles ALTER COLUMN id SET DEFAULT nextval('public.profiles_id_seq'::regclass);


--
-- Name: programs id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.programs ALTER COLUMN id SET DEFAULT nextval('public.programs_id_seq'::regclass);


--
-- Name: promoters id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.promoters ALTER COLUMN id SET DEFAULT nextval('public.promoters_id_seq'::regclass);


--
-- Name: property_types id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.property_types ALTER COLUMN id SET DEFAULT nextval('public.property_types_id_seq'::regclass);


--
-- Name: queries id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.queries ALTER COLUMN id SET DEFAULT nextval('public.queries_id_seq'::regclass);


--
-- Name: referral_codes id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.referral_codes ALTER COLUMN id SET DEFAULT nextval('public.referral_codes_id_seq'::regclass);


--
-- Name: referral_types id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.referral_types ALTER COLUMN id SET DEFAULT nextval('public.referral_types_id_seq'::regclass);


--
-- Name: referrer_branch_cities id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.referrer_branch_cities ALTER COLUMN id SET DEFAULT nextval('public.referrer_branch_cities_id_seq'::regclass);


--
-- Name: roles id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.roles ALTER COLUMN id SET DEFAULT nextval('public.roles_id_seq'::regclass);


--
-- Name: shareholdings id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.shareholdings ALTER COLUMN id SET DEFAULT nextval('public.shareholdings_id_seq'::regclass);


--
-- Name: signup_requests id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.signup_requests ALTER COLUMN id SET DEFAULT nextval('public.signup_requests_id_seq'::regclass);


--
-- Name: tata_apis id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tata_apis ALTER COLUMN id SET DEFAULT nextval('public.tata_apis_id_seq'::regclass);


--
-- Name: users id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass);


--
-- Name: versions id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.versions ALTER COLUMN id SET DEFAULT nextval('public.versions_id_seq'::regclass);


--
-- Name: databases databases_pkey; Type: CONSTRAINT; Schema: _sqlx_test; Owner: nikhil
--

ALTER TABLE ONLY _sqlx_test.databases
    ADD CONSTRAINT databases_pkey PRIMARY KEY (db_name);


--
-- Name: hdb_action_log hdb_action_log_pkey; Type: CONSTRAINT; Schema: hdb_catalog; Owner: postgres
--

ALTER TABLE ONLY hdb_catalog.hdb_action_log
    ADD CONSTRAINT hdb_action_log_pkey PRIMARY KEY (id);


--
-- Name: hdb_cron_event_invocation_logs hdb_cron_event_invocation_logs_pkey; Type: CONSTRAINT; Schema: hdb_catalog; Owner: postgres
--

ALTER TABLE ONLY hdb_catalog.hdb_cron_event_invocation_logs
    ADD CONSTRAINT hdb_cron_event_invocation_logs_pkey PRIMARY KEY (id);


--
-- Name: hdb_cron_events hdb_cron_events_pkey; Type: CONSTRAINT; Schema: hdb_catalog; Owner: postgres
--

ALTER TABLE ONLY hdb_catalog.hdb_cron_events
    ADD CONSTRAINT hdb_cron_events_pkey PRIMARY KEY (id);


--
-- Name: hdb_metadata hdb_metadata_pkey; Type: CONSTRAINT; Schema: hdb_catalog; Owner: postgres
--

ALTER TABLE ONLY hdb_catalog.hdb_metadata
    ADD CONSTRAINT hdb_metadata_pkey PRIMARY KEY (id);


--
-- Name: hdb_metadata hdb_metadata_resource_version_key; Type: CONSTRAINT; Schema: hdb_catalog; Owner: postgres
--

ALTER TABLE ONLY hdb_catalog.hdb_metadata
    ADD CONSTRAINT hdb_metadata_resource_version_key UNIQUE (resource_version);


--
-- Name: hdb_scheduled_event_invocation_logs hdb_scheduled_event_invocation_logs_pkey; Type: CONSTRAINT; Schema: hdb_catalog; Owner: postgres
--

ALTER TABLE ONLY hdb_catalog.hdb_scheduled_event_invocation_logs
    ADD CONSTRAINT hdb_scheduled_event_invocation_logs_pkey PRIMARY KEY (id);


--
-- Name: hdb_scheduled_events hdb_scheduled_events_pkey; Type: CONSTRAINT; Schema: hdb_catalog; Owner: postgres
--

ALTER TABLE ONLY hdb_catalog.hdb_scheduled_events
    ADD CONSTRAINT hdb_scheduled_events_pkey PRIMARY KEY (id);


--
-- Name: hdb_schema_notifications hdb_schema_notifications_pkey; Type: CONSTRAINT; Schema: hdb_catalog; Owner: postgres
--

ALTER TABLE ONLY hdb_catalog.hdb_schema_notifications
    ADD CONSTRAINT hdb_schema_notifications_pkey PRIMARY KEY (id);


--
-- Name: hdb_version hdb_version_pkey; Type: CONSTRAINT; Schema: hdb_catalog; Owner: postgres
--

ALTER TABLE ONLY hdb_catalog.hdb_version
    ADD CONSTRAINT hdb_version_pkey PRIMARY KEY (hasura_uuid);


--
-- Name: _sqlx_migrations _sqlx_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: nikhil
--

ALTER TABLE ONLY public._sqlx_migrations
    ADD CONSTRAINT _sqlx_migrations_pkey PRIMARY KEY (version);


--
-- Name: active_storage_attachments active_storage_attachments_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.active_storage_attachments
    ADD CONSTRAINT active_storage_attachments_pkey PRIMARY KEY (id);


--
-- Name: active_storage_blobs active_storage_blobs_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.active_storage_blobs
    ADD CONSTRAINT active_storage_blobs_pkey PRIMARY KEY (id);


--
-- Name: active_storage_variant_records active_storage_variant_records_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.active_storage_variant_records
    ADD CONSTRAINT active_storage_variant_records_pkey PRIMARY KEY (id);


--
-- Name: activity_logs activity_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.activity_logs
    ADD CONSTRAINT activity_logs_pkey PRIMARY KEY (id);


--
-- Name: admin_users admin_users_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.admin_users
    ADD CONSTRAINT admin_users_pkey PRIMARY KEY (id);


--
-- Name: api_request_logs api_request_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.api_request_logs
    ADD CONSTRAINT api_request_logs_pkey PRIMARY KEY (id);


--
-- Name: application_statuses application_statuses_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.application_statuses
    ADD CONSTRAINT application_statuses_pkey PRIMARY KEY (id);


--
-- Name: ar_internal_metadata ar_internal_metadata_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.ar_internal_metadata
    ADD CONSTRAINT ar_internal_metadata_pkey PRIMARY KEY (key);


--
-- Name: bank_statements bank_statements_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.bank_statements
    ADD CONSTRAINT bank_statements_pkey PRIMARY KEY (id);


--
-- Name: banking_histories banking_histories_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.banking_histories
    ADD CONSTRAINT banking_histories_pkey PRIMARY KEY (id);


--
-- Name: borrower_infos borrower_infos_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.borrower_infos
    ADD CONSTRAINT borrower_infos_pkey PRIMARY KEY (id);


--
-- Name: borrower_profiles borrower_profiles_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.borrower_profiles
    ADD CONSTRAINT borrower_profiles_pkey PRIMARY KEY (id);


--
-- Name: business_metas business_metas_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.business_metas
    ADD CONSTRAINT business_metas_pkey PRIMARY KEY (id);


--
-- Name: business_types business_types_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.business_types
    ADD CONSTRAINT business_types_pkey PRIMARY KEY (id);


--
-- Name: businesses businesses_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.businesses
    ADD CONSTRAINT businesses_pkey PRIMARY KEY (id);


--
-- Name: carts carts_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.carts
    ADD CONSTRAINT carts_pkey PRIMARY KEY (id);


--
-- Name: collaterals collaterals_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.collaterals
    ADD CONSTRAINT collaterals_pkey PRIMARY KEY (id);


--
-- Name: comments comments_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.comments
    ADD CONSTRAINT comments_pkey PRIMARY KEY (id);


--
-- Name: credit_reports credit_reports_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.credit_reports
    ADD CONSTRAINT credit_reports_pkey PRIMARY KEY (id);


--
-- Name: designations designations_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.designations
    ADD CONSTRAINT designations_pkey PRIMARY KEY (id);


--
-- Name: directors directors_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.directors
    ADD CONSTRAINT directors_pkey PRIMARY KEY (id);


--
-- Name: disbursements disbursements_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.disbursements
    ADD CONSTRAINT disbursements_pkey PRIMARY KEY (id);


--
-- Name: document_types document_types_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.document_types
    ADD CONSTRAINT document_types_pkey PRIMARY KEY (id);


--
-- Name: documents documents_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_pkey PRIMARY KEY (id);


--
-- Name: employee_profiles employee_profiles_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.employee_profiles
    ADD CONSTRAINT employee_profiles_pkey PRIMARY KEY (id);


--
-- Name: finance_records finance_records_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.finance_records
    ADD CONSTRAINT finance_records_pkey PRIMARY KEY (id);


--
-- Name: financial_ratios financial_ratios_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.financial_ratios
    ADD CONSTRAINT financial_ratios_pkey PRIMARY KEY (id);


--
-- Name: financial_summaries financial_summaries_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.financial_summaries
    ADD CONSTRAINT financial_summaries_pkey PRIMARY KEY (id);


--
-- Name: gst_basic_details gst_basic_details_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.gst_basic_details
    ADD CONSTRAINT gst_basic_details_pkey PRIMARY KEY (id);


--
-- Name: gst_reports gst_reports_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.gst_reports
    ADD CONSTRAINT gst_reports_pkey PRIMARY KEY (id);


--
-- Name: gst_returns gst_returns_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.gst_returns
    ADD CONSTRAINT gst_returns_pkey PRIMARY KEY (id);


--
-- Name: industry_types industry_types_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.industry_types
    ADD CONSTRAINT industry_types_pkey PRIMARY KEY (id);


--
-- Name: institution_types institution_types_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.institution_types
    ADD CONSTRAINT institution_types_pkey PRIMARY KEY (id);


--
-- Name: institutions institutions_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.institutions
    ADD CONSTRAINT institutions_pkey PRIMARY KEY (id);


--
-- Name: instrument_relationships instrument_relationships_pkey; Type: CONSTRAINT; Schema: public; Owner: nikhil
--

ALTER TABLE ONLY public.instrument_relationships
    ADD CONSTRAINT instrument_relationships_pkey PRIMARY KEY (id);


--
-- Name: instrument_relationships instrument_relationships_source_instrument_id_target_instru_key; Type: CONSTRAINT; Schema: public; Owner: nikhil
--

ALTER TABLE ONLY public.instrument_relationships
    ADD CONSTRAINT instrument_relationships_source_instrument_id_target_instru_key UNIQUE (source_instrument_id, target_instrument_id, relationship_type);


--
-- Name: instruments instruments_pkey; Type: CONSTRAINT; Schema: public; Owner: nikhil
--

ALTER TABLE ONLY public.instruments
    ADD CONSTRAINT instruments_pkey PRIMARY KEY (id);


--
-- Name: itr_reports itr_reports_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.itr_reports
    ADD CONSTRAINT itr_reports_pkey PRIMARY KEY (id);


--
-- Name: jwt_denylist jwt_denylist_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.jwt_denylist
    ADD CONSTRAINT jwt_denylist_pkey PRIMARY KEY (id);


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
-- Name: leads leads_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.leads
    ADD CONSTRAINT leads_pkey PRIMARY KEY (id);


--
-- Name: line_items line_items_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.line_items
    ADD CONSTRAINT line_items_pkey PRIMARY KEY (id);


--
-- Name: loan_applications loan_applications_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.loan_applications
    ADD CONSTRAINT loan_applications_pkey PRIMARY KEY (id);


--
-- Name: loan_matches loan_matches_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.loan_matches
    ADD CONSTRAINT loan_matches_pkey PRIMARY KEY (id);


--
-- Name: loan_types loan_types_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.loan_types
    ADD CONSTRAINT loan_types_pkey PRIMARY KEY (id);


--
-- Name: locations locations_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.locations
    ADD CONSTRAINT locations_pkey PRIMARY KEY (id);


--
-- Name: meetings meetings_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.meetings
    ADD CONSTRAINT meetings_pkey PRIMARY KEY (id);


--
-- Name: observations observations_pkey; Type: CONSTRAINT; Schema: public; Owner: nikhil
--

ALTER TABLE ONLY public.observations
    ADD CONSTRAINT observations_pkey PRIMARY KEY (id);


--
-- Name: product_details product_details_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.product_details
    ADD CONSTRAINT product_details_pkey PRIMARY KEY (id);


--
-- Name: product_types product_types_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.product_types
    ADD CONSTRAINT product_types_pkey PRIMARY KEY (id);


--
-- Name: products products_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_pkey PRIMARY KEY (id);


--
-- Name: profile_views profile_views_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.profile_views
    ADD CONSTRAINT profile_views_pkey PRIMARY KEY (id);


--
-- Name: profiles profiles_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.profiles
    ADD CONSTRAINT profiles_pkey PRIMARY KEY (id);


--
-- Name: programs programs_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.programs
    ADD CONSTRAINT programs_pkey PRIMARY KEY (id);


--
-- Name: promoters promoters_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.promoters
    ADD CONSTRAINT promoters_pkey PRIMARY KEY (id);


--
-- Name: property_types property_types_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.property_types
    ADD CONSTRAINT property_types_pkey PRIMARY KEY (id);


--
-- Name: queries queries_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.queries
    ADD CONSTRAINT queries_pkey PRIMARY KEY (id);


--
-- Name: referral_codes referral_codes_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.referral_codes
    ADD CONSTRAINT referral_codes_pkey PRIMARY KEY (id);


--
-- Name: referral_types referral_types_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.referral_types
    ADD CONSTRAINT referral_types_pkey PRIMARY KEY (id);


--
-- Name: referrer_branch_cities referrer_branch_cities_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.referrer_branch_cities
    ADD CONSTRAINT referrer_branch_cities_pkey PRIMARY KEY (id);


--
-- Name: research_sessions research_sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: nikhil
--

ALTER TABLE ONLY public.research_sessions
    ADD CONSTRAINT research_sessions_pkey PRIMARY KEY (id);


--
-- Name: roles roles_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.roles
    ADD CONSTRAINT roles_pkey PRIMARY KEY (id);


--
-- Name: schema_migrations schema_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.schema_migrations
    ADD CONSTRAINT schema_migrations_pkey PRIMARY KEY (version);


--
-- Name: shareholdings shareholdings_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.shareholdings
    ADD CONSTRAINT shareholdings_pkey PRIMARY KEY (id);


--
-- Name: signup_requests signup_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.signup_requests
    ADD CONSTRAINT signup_requests_pkey PRIMARY KEY (id);


--
-- Name: tata_apis tata_apis_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tata_apis
    ADD CONSTRAINT tata_apis_pkey PRIMARY KEY (id);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: versions versions_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.versions
    ADD CONSTRAINT versions_pkey PRIMARY KEY (id);


--
-- Name: databases_created_at; Type: INDEX; Schema: _sqlx_test; Owner: nikhil
--

CREATE INDEX databases_created_at ON _sqlx_test.databases USING btree (created_at);


--
-- Name: hdb_cron_event_invocation_event_id; Type: INDEX; Schema: hdb_catalog; Owner: postgres
--

CREATE INDEX hdb_cron_event_invocation_event_id ON hdb_catalog.hdb_cron_event_invocation_logs USING btree (event_id);


--
-- Name: hdb_cron_event_status; Type: INDEX; Schema: hdb_catalog; Owner: postgres
--

CREATE INDEX hdb_cron_event_status ON hdb_catalog.hdb_cron_events USING btree (status);


--
-- Name: hdb_cron_events_unique_scheduled; Type: INDEX; Schema: hdb_catalog; Owner: postgres
--

CREATE UNIQUE INDEX hdb_cron_events_unique_scheduled ON hdb_catalog.hdb_cron_events USING btree (trigger_name, scheduled_time) WHERE (status = 'scheduled'::text);


--
-- Name: hdb_scheduled_event_status; Type: INDEX; Schema: hdb_catalog; Owner: postgres
--

CREATE INDEX hdb_scheduled_event_status ON hdb_catalog.hdb_scheduled_events USING btree (status);


--
-- Name: hdb_version_one_row; Type: INDEX; Schema: hdb_catalog; Owner: postgres
--

CREATE UNIQUE INDEX hdb_version_one_row ON hdb_catalog.hdb_version USING btree (((version IS NOT NULL)));


--
-- Name: gst_basic_details_query; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX gst_basic_details_query ON public.gst_basic_details_queries USING btree (gst_basic_detail_id, query_id);


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
-- Name: idx_observations_instrument; Type: INDEX; Schema: public; Owner: nikhil
--

CREATE INDEX idx_observations_instrument ON public.observations USING btree (instrument_id);


--
-- Name: idx_observations_observed_at; Type: INDEX; Schema: public; Owner: nikhil
--

CREATE INDEX idx_observations_observed_at ON public.observations USING btree (observed_at);


--
-- Name: idx_observations_type; Type: INDEX; Schema: public; Owner: nikhil
--

CREATE INDEX idx_observations_type ON public.observations USING btree (observation_type);


--
-- Name: index_active_storage_attachments_on_blob_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_active_storage_attachments_on_blob_id ON public.active_storage_attachments USING btree (blob_id);


--
-- Name: index_active_storage_attachments_uniqueness; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX index_active_storage_attachments_uniqueness ON public.active_storage_attachments USING btree (record_type, record_id, name, blob_id);


--
-- Name: index_active_storage_blobs_on_key; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX index_active_storage_blobs_on_key ON public.active_storage_blobs USING btree (key);


--
-- Name: index_active_storage_variant_records_uniqueness; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX index_active_storage_variant_records_uniqueness ON public.active_storage_variant_records USING btree (blob_id, variation_digest);


--
-- Name: index_activity_logs_on_relatable_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_activity_logs_on_relatable_id ON public.activity_logs USING btree (relatable_id);


--
-- Name: index_activity_logs_on_relatable_type; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_activity_logs_on_relatable_type ON public.activity_logs USING btree (relatable_type);


--
-- Name: index_admin_users_on_deleted_at; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_admin_users_on_deleted_at ON public.admin_users USING btree (deleted_at);


--
-- Name: index_admin_users_on_email; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX index_admin_users_on_email ON public.admin_users USING btree (email);


--
-- Name: index_admin_users_on_invitation_token; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX index_admin_users_on_invitation_token ON public.admin_users USING btree (invitation_token);


--
-- Name: index_admin_users_on_invited_by; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_admin_users_on_invited_by ON public.admin_users USING btree (invited_by_type, invited_by_id);


--
-- Name: index_admin_users_on_invited_by_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_admin_users_on_invited_by_id ON public.admin_users USING btree (invited_by_id);


--
-- Name: index_admin_users_on_referral_code; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX index_admin_users_on_referral_code ON public.admin_users USING btree (referral_code);


--
-- Name: index_admin_users_on_reset_password_token; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX index_admin_users_on_reset_password_token ON public.admin_users USING btree (reset_password_token);


--
-- Name: index_api_request_logs_on_log_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_api_request_logs_on_log_id ON public.api_request_logs USING btree (log_id);


--
-- Name: index_application_statuses_on_deleted_at; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_application_statuses_on_deleted_at ON public.application_statuses USING btree (deleted_at);


--
-- Name: index_bank_statements_on_borrower_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_bank_statements_on_borrower_id ON public.bank_statements USING btree (borrower_id);


--
-- Name: index_bank_statements_on_business_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_bank_statements_on_business_id ON public.bank_statements USING btree (business_id);


--
-- Name: index_bank_statements_on_deleted_at; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_bank_statements_on_deleted_at ON public.bank_statements USING btree (deleted_at);


--
-- Name: index_bank_statements_on_entity_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_bank_statements_on_entity_id ON public.bank_statements USING btree (entity_id);


--
-- Name: index_bank_statements_on_link_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_bank_statements_on_link_id ON public.bank_statements USING btree (link_id);


--
-- Name: index_borrower_infos_on_borrower_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_borrower_infos_on_borrower_id ON public.borrower_infos USING btree (borrower_id);


--
-- Name: index_borrower_infos_on_full_name; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_borrower_infos_on_full_name ON public.borrower_infos USING btree (full_name);


--
-- Name: index_borrower_infos_on_pan_number; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_borrower_infos_on_pan_number ON public.borrower_infos USING btree (pan_number);


--
-- Name: index_borrower_profiles_on_institution_product_type; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_borrower_profiles_on_institution_product_type ON public.borrower_profiles USING btree (institution_id, product_type_id, business_id);


--
-- Name: index_business_metas_on_business_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_business_metas_on_business_id ON public.business_metas USING btree (business_id);


--
-- Name: index_business_metas_on_data; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_business_metas_on_data ON public.business_metas USING gin (data);


--
-- Name: index_business_types_on_deleted_at; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_business_types_on_deleted_at ON public.business_types USING btree (deleted_at);


--
-- Name: index_businesses_on_business_type_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_businesses_on_business_type_id ON public.businesses USING btree (business_type_id);


--
-- Name: index_businesses_on_employee_profile_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_businesses_on_employee_profile_id ON public.businesses USING btree (employee_profile_id);


--
-- Name: index_businesses_on_industry_type_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_businesses_on_industry_type_id ON public.businesses USING btree (industry_type_id);


--
-- Name: index_collaterals_on_business_users; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_collaterals_on_business_users ON public.collaterals USING btree (property_type_id, possession_type_id, business_id);


--
-- Name: index_comments_on_commentable_id_and_commentable_type; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_comments_on_commentable_id_and_commentable_type ON public.comments USING btree (commentable_id, commentable_type);


--
-- Name: index_comments_on_loan_application_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_comments_on_loan_application_id ON public.comments USING btree (loan_application_id);


--
-- Name: index_credit_reports_on_score; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_credit_reports_on_score ON public.credit_reports USING btree (score);


--
-- Name: index_directors_on_business_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_directors_on_business_id ON public.directors USING btree (business_id);


--
-- Name: index_directors_on_business_id_and_din; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX index_directors_on_business_id_and_din ON public.directors USING btree (business_id, din);


--
-- Name: index_document_types_on_deleted_at; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_document_types_on_deleted_at ON public.document_types USING btree (deleted_at);


--
-- Name: index_documents_on_document_type; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_documents_on_document_type ON public.documents USING btree (document_type_id, business_id);


--
-- Name: index_documents_on_document_type_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_documents_on_document_type_id ON public.documents USING btree (document_type_id);


--
-- Name: index_employee_profiles_on_admin_user_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_employee_profiles_on_admin_user_id ON public.employee_profiles USING btree (admin_user_id);


--
-- Name: index_employee_profiles_on_location_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_employee_profiles_on_location_id ON public.employee_profiles USING btree (location_id);


--
-- Name: index_financial_ratios_on_business_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_financial_ratios_on_business_id ON public.financial_ratios USING btree (business_id);


--
-- Name: index_financial_ratios_on_business_id_and_year; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX index_financial_ratios_on_business_id_and_year ON public.financial_ratios USING btree (business_id, year);


--
-- Name: index_financial_summaries_on_business_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_financial_summaries_on_business_id ON public.financial_summaries USING btree (business_id);


--
-- Name: index_financial_summaries_on_business_id_and_year; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX index_financial_summaries_on_business_id_and_year ON public.financial_summaries USING btree (business_id, year);


--
-- Name: index_for_banking_histories; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX index_for_banking_histories ON public.banking_histories USING btree (business_id, year, month, day);


--
-- Name: index_for_business_id_and_employee_profile_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_for_business_id_and_employee_profile_id ON public.businesses_employee_profiles USING btree (business_id, employee_profile_id);


--
-- Name: index_for_employee_profile_id_and_business_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_for_employee_profile_id_and_business_id ON public.businesses_employee_profiles USING btree (employee_profile_id, business_id);


--
-- Name: index_for_loan_applications_employee_profiles; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_for_loan_applications_employee_profiles ON public.employee_profiles_loan_applications USING btree (loan_application_id, employee_profile_id);


--
-- Name: index_gst_basic_details_on_borrower_id_and_business_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_gst_basic_details_on_borrower_id_and_business_id ON public.gst_basic_details USING btree (borrower_id, business_id);


--
-- Name: index_gst_basic_details_on_gstin; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_gst_basic_details_on_gstin ON public.gst_basic_details USING btree (gstin);


--
-- Name: index_gst_reports_on_borrower_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_gst_reports_on_borrower_id ON public.gst_reports USING btree (borrower_id);


--
-- Name: index_gst_reports_on_business_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_gst_reports_on_business_id ON public.gst_reports USING btree (business_id);


--
-- Name: index_gst_reports_on_deleted_at; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_gst_reports_on_deleted_at ON public.gst_reports USING btree (deleted_at);


--
-- Name: index_gst_reports_on_gst_basic_detail_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_gst_reports_on_gst_basic_detail_id ON public.gst_reports USING btree (gst_basic_detail_id);


--
-- Name: index_gst_reports_on_gstin; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_gst_reports_on_gstin ON public.gst_reports USING btree (gstin);


--
-- Name: index_gst_reports_on_reference_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_gst_reports_on_reference_id ON public.gst_reports USING btree (reference_id);


--
-- Name: index_gst_returns_on_business_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_gst_returns_on_business_id ON public.gst_returns USING btree (business_id);


--
-- Name: index_gst_returns_on_year_and_month_and_business_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_gst_returns_on_year_and_month_and_business_id ON public.gst_returns USING btree (year, month, business_id);


--
-- Name: index_industry_types_on_deleted_at; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_industry_types_on_deleted_at ON public.industry_types USING btree (deleted_at);


--
-- Name: index_institution_types_on_deleted_at; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_institution_types_on_deleted_at ON public.institution_types USING btree (deleted_at);


--
-- Name: index_institutions_on_deleted_at; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_institutions_on_deleted_at ON public.institutions USING btree (deleted_at);


--
-- Name: index_institutions_on_institution_type_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_institutions_on_institution_type_id ON public.institutions USING btree (institution_type_id);


--
-- Name: index_institutions_on_location_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_institutions_on_location_id ON public.institutions USING btree (location_id);


--
-- Name: index_itr_reports_on_deleted_at; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_itr_reports_on_deleted_at ON public.itr_reports USING btree (deleted_at);


--
-- Name: index_itr_reports_queries_on_itr_report_id_and_query_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_itr_reports_queries_on_itr_report_id_and_query_id ON public.itr_reports_queries USING btree (itr_report_id, query_id);


--
-- Name: index_itr_reports_queries_on_query_id_and_itr_report_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_itr_reports_queries_on_query_id_and_itr_report_id ON public.itr_reports_queries USING btree (query_id, itr_report_id);


--
-- Name: index_jwt_denylist_on_jti; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_jwt_denylist_on_jti ON public.jwt_denylist USING btree (jti);


--
-- Name: index_loan_applications_on_api_request_log_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_loan_applications_on_api_request_log_id ON public.loan_applications USING btree (api_request_log_id);


--
-- Name: index_loan_applications_on_business_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_loan_applications_on_business_id ON public.loan_applications USING btree (business_id);


--
-- Name: index_loan_applications_on_deleted_at; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_loan_applications_on_deleted_at ON public.loan_applications USING btree (deleted_at);


--
-- Name: index_loan_applications_on_product_type_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_loan_applications_on_product_type_id ON public.loan_applications USING btree (product_type_id);


--
-- Name: index_loan_applications_on_property_type_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_loan_applications_on_property_type_id ON public.loan_applications USING btree (property_type_id);


--
-- Name: index_loan_matches_on_deleted_at; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_loan_matches_on_deleted_at ON public.loan_matches USING btree (deleted_at);


--
-- Name: index_loan_matches_on_loan_application_id_and_product_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_loan_matches_on_loan_application_id_and_product_id ON public.loan_matches USING btree (loan_application_id, product_id);


--
-- Name: index_locations_on_deleted_at; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_locations_on_deleted_at ON public.locations USING btree (deleted_at);


--
-- Name: index_meetings_on_loan_match_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_meetings_on_loan_match_id ON public.meetings USING btree (loan_match_id);


--
-- Name: index_product_property_types_on_property_product_type; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_product_property_types_on_property_product_type ON public.product_types_property_types USING btree (product_type_id, property_type_id);


--
-- Name: index_product_types_on_deleted_at; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_product_types_on_deleted_at ON public.product_types USING btree (deleted_at);


--
-- Name: index_product_types_property_types_on_product_type_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_product_types_property_types_on_product_type_id ON public.product_types_property_types USING btree (product_type_id);


--
-- Name: index_product_types_property_types_on_property_type_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_product_types_property_types_on_property_type_id ON public.product_types_property_types USING btree (property_type_id);


--
-- Name: index_products_on_deleted_at; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_products_on_deleted_at ON public.products USING btree (deleted_at);


--
-- Name: index_products_on_loan_types; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_products_on_loan_types ON public.products USING gin (loan_types);


--
-- Name: index_profiles_on_deleted_at; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_profiles_on_deleted_at ON public.profiles USING btree (deleted_at);


--
-- Name: index_profiles_on_employee_profile_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_profiles_on_employee_profile_id ON public.profiles USING btree (employee_profile_id);


--
-- Name: index_profiles_on_institution_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_profiles_on_institution_id ON public.profiles USING btree (institution_id);


--
-- Name: index_profiles_on_lender_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_profiles_on_lender_id ON public.profiles USING btree (lender_id);


--
-- Name: index_programs_on_deleted_at; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_programs_on_deleted_at ON public.programs USING btree (deleted_at);


--
-- Name: index_promoters_on_business_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_promoters_on_business_id ON public.promoters USING btree (business_id);


--
-- Name: index_property_types_on_parent_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_property_types_on_parent_id ON public.property_types USING btree (parent_id);


--
-- Name: index_roles_on_admin_user_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_roles_on_admin_user_id ON public.roles USING btree (admin_user_id);


--
-- Name: index_roles_on_designation_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_roles_on_designation_id ON public.roles USING btree (designation_id);


--
-- Name: index_users_on_confirmation_token; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX index_users_on_confirmation_token ON public.users USING btree (confirmation_token);


--
-- Name: index_users_on_deleted_at; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_users_on_deleted_at ON public.users USING btree (deleted_at);


--
-- Name: index_users_on_invitation_token; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX index_users_on_invitation_token ON public.users USING btree (invitation_token);


--
-- Name: index_users_on_invited_by; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_users_on_invited_by ON public.users USING btree (invited_by_type, invited_by_id);


--
-- Name: index_users_on_invited_by_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_users_on_invited_by_id ON public.users USING btree (invited_by_id);


--
-- Name: index_users_on_mobile_number; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX index_users_on_mobile_number ON public.users USING btree (mobile_number);


--
-- Name: index_users_on_referral_code; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX index_users_on_referral_code ON public.users USING btree (referral_code);


--
-- Name: index_users_on_reset_password_token; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX index_users_on_reset_password_token ON public.users USING btree (reset_password_token);


--
-- Name: index_versions_on_item_type_and_item_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX index_versions_on_item_type_and_item_id ON public.versions USING btree (item_type, item_id);


--
-- Name: query_gst_basic_details; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX query_gst_basic_details ON public.gst_basic_details_queries USING btree (query_id, gst_basic_detail_id);


--
-- Name: hdb_cron_event_invocation_logs hdb_cron_event_invocation_logs_event_id_fkey; Type: FK CONSTRAINT; Schema: hdb_catalog; Owner: postgres
--

ALTER TABLE ONLY hdb_catalog.hdb_cron_event_invocation_logs
    ADD CONSTRAINT hdb_cron_event_invocation_logs_event_id_fkey FOREIGN KEY (event_id) REFERENCES hdb_catalog.hdb_cron_events(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: hdb_scheduled_event_invocation_logs hdb_scheduled_event_invocation_logs_event_id_fkey; Type: FK CONSTRAINT; Schema: hdb_catalog; Owner: postgres
--

ALTER TABLE ONLY hdb_catalog.hdb_scheduled_event_invocation_logs
    ADD CONSTRAINT hdb_scheduled_event_invocation_logs_event_id_fkey FOREIGN KEY (event_id) REFERENCES hdb_catalog.hdb_scheduled_events(id) ON UPDATE CASCADE ON DELETE CASCADE;


--
-- Name: promoters fk_rails_023d2982e6; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.promoters
    ADD CONSTRAINT fk_rails_023d2982e6 FOREIGN KEY (business_id) REFERENCES public.businesses(id);


--
-- Name: businesses fk_rails_19428e368c; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.businesses
    ADD CONSTRAINT fk_rails_19428e368c FOREIGN KEY (industry_type_id) REFERENCES public.industry_types(id);


--
-- Name: employee_profiles fk_rails_3da73d07bb; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.employee_profiles
    ADD CONSTRAINT fk_rails_3da73d07bb FOREIGN KEY (location_id) REFERENCES public.locations(id);


--
-- Name: loan_applications fk_rails_3eb57594a9; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.loan_applications
    ADD CONSTRAINT fk_rails_3eb57594a9 FOREIGN KEY (property_type_id) REFERENCES public.property_types(id);


--
-- Name: gst_returns fk_rails_6c5431cf2a; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.gst_returns
    ADD CONSTRAINT fk_rails_6c5431cf2a FOREIGN KEY (business_id) REFERENCES public.businesses(id);


--
-- Name: property_types fk_rails_7bfbf8423c; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.property_types
    ADD CONSTRAINT fk_rails_7bfbf8423c FOREIGN KEY (parent_id) REFERENCES public.property_types(id);


--
-- Name: profiles fk_rails_7ee10e9afb; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.profiles
    ADD CONSTRAINT fk_rails_7ee10e9afb FOREIGN KEY (lender_id) REFERENCES public.users(id);


--
-- Name: loan_applications fk_rails_882f44bab5; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.loan_applications
    ADD CONSTRAINT fk_rails_882f44bab5 FOREIGN KEY (product_type_id) REFERENCES public.product_types(id);


--
-- Name: profiles fk_rails_8adb6b5ce0; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.profiles
    ADD CONSTRAINT fk_rails_8adb6b5ce0 FOREIGN KEY (institution_id) REFERENCES public.institutions(id);


--
-- Name: financial_summaries fk_rails_90e2e344dc; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.financial_summaries
    ADD CONSTRAINT fk_rails_90e2e344dc FOREIGN KEY (business_id) REFERENCES public.businesses(id);


--
-- Name: roles fk_rails_93c81b7209; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.roles
    ADD CONSTRAINT fk_rails_93c81b7209 FOREIGN KEY (admin_user_id) REFERENCES public.admin_users(id);


--
-- Name: active_storage_variant_records fk_rails_993965df05; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.active_storage_variant_records
    ADD CONSTRAINT fk_rails_993965df05 FOREIGN KEY (blob_id) REFERENCES public.active_storage_blobs(id);


--
-- Name: institutions fk_rails_a0518b6c70; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.institutions
    ADD CONSTRAINT fk_rails_a0518b6c70 FOREIGN KEY (location_id) REFERENCES public.locations(id);


--
-- Name: meetings fk_rails_a21d8071b3; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.meetings
    ADD CONSTRAINT fk_rails_a21d8071b3 FOREIGN KEY (loan_match_id) REFERENCES public.loan_matches(id);


--
-- Name: institutions fk_rails_a4be856fe1; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.institutions
    ADD CONSTRAINT fk_rails_a4be856fe1 FOREIGN KEY (institution_type_id) REFERENCES public.institution_types(id);


--
-- Name: businesses fk_rails_a52c2e29c7; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.businesses
    ADD CONSTRAINT fk_rails_a52c2e29c7 FOREIGN KEY (employee_profile_id) REFERENCES public.employee_profiles(id);


--
-- Name: roles fk_rails_a5ae70a8a2; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.roles
    ADD CONSTRAINT fk_rails_a5ae70a8a2 FOREIGN KEY (designation_id) REFERENCES public.designations(id);


--
-- Name: directors fk_rails_ae3cb97915; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.directors
    ADD CONSTRAINT fk_rails_ae3cb97915 FOREIGN KEY (business_id) REFERENCES public.businesses(id);


--
-- Name: business_metas fk_rails_bec66b5b4f; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.business_metas
    ADD CONSTRAINT fk_rails_bec66b5b4f FOREIGN KEY (business_id) REFERENCES public.businesses(id);


--
-- Name: active_storage_attachments fk_rails_c3b3935057; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.active_storage_attachments
    ADD CONSTRAINT fk_rails_c3b3935057 FOREIGN KEY (blob_id) REFERENCES public.active_storage_blobs(id);


--
-- Name: financial_ratios fk_rails_d6ccb79da6; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.financial_ratios
    ADD CONSTRAINT fk_rails_d6ccb79da6 FOREIGN KEY (business_id) REFERENCES public.businesses(id);


--
-- Name: businesses fk_rails_d6f272cd54; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.businesses
    ADD CONSTRAINT fk_rails_d6f272cd54 FOREIGN KEY (business_type_id) REFERENCES public.business_types(id);


--
-- Name: loan_applications fk_rails_e79058ad34; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.loan_applications
    ADD CONSTRAINT fk_rails_e79058ad34 FOREIGN KEY (business_id) REFERENCES public.businesses(id);


--
-- Name: profiles fk_rails_f763ad965b; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.profiles
    ADD CONSTRAINT fk_rails_f763ad965b FOREIGN KEY (employee_profile_id) REFERENCES public.employee_profiles(id);


--
-- Name: instrument_relationships instrument_relationships_source_instrument_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nikhil
--

ALTER TABLE ONLY public.instrument_relationships
    ADD CONSTRAINT instrument_relationships_source_instrument_id_fkey FOREIGN KEY (source_instrument_id) REFERENCES public.instruments(id);


--
-- Name: instrument_relationships instrument_relationships_target_instrument_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nikhil
--

ALTER TABLE ONLY public.instrument_relationships
    ADD CONSTRAINT instrument_relationships_target_instrument_id_fkey FOREIGN KEY (target_instrument_id) REFERENCES public.instruments(id);


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
-- Name: observations observations_instrument_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: nikhil
--

ALTER TABLE ONLY public.observations
    ADD CONSTRAINT observations_instrument_id_fkey FOREIGN KEY (instrument_id) REFERENCES public.instruments(id);


--
-- Name: SCHEMA public; Type: ACL; Schema: -; Owner: postgres
--

REVOKE ALL ON SCHEMA public FROM nikhil;
REVOKE ALL ON SCHEMA public FROM PUBLIC;
GRANT ALL ON SCHEMA public TO postgres;
GRANT ALL ON SCHEMA public TO PUBLIC;


--
-- PostgreSQL database dump complete
--

