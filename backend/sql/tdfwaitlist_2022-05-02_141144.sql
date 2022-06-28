--
-- PostgreSQL database dump
--

-- Dumped from database version 13.6 (Debian 13.6-1.pgdg110+1)
-- Dumped by pg_dump version 14.2 (Debian 14.2-1.pgdg110+1)

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

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: access_token; Type: TABLE; Schema: public; Owner: tdfwaitlist
--

CREATE TABLE public.access_token (
    character_id bigint NOT NULL,
    access_token character varying(4096) NOT NULL,
    expires bigint NOT NULL,
    scopes character varying(1024) NOT NULL
);


ALTER TABLE public.access_token OWNER TO tdfwaitlist;

--
-- Name: access_token_esi; Type: TABLE; Schema: public; Owner: tdfwaitlist
--

CREATE TABLE public.access_token_esi (
    character_id bigint NOT NULL,
    access_token character varying(4096) NOT NULL,
    expires bigint NOT NULL,
    scopes character varying(1024) NOT NULL
);


ALTER TABLE public.access_token_esi OWNER TO tdfwaitlist;

--
-- Name: admins; Type: TABLE; Schema: public; Owner: tdfwaitlist
--

CREATE TABLE public.admins (
    character_id bigint NOT NULL,
    level character varying(64) NOT NULL
);


ALTER TABLE public.admins OWNER TO tdfwaitlist;

--
-- Name: alt_character; Type: TABLE; Schema: public; Owner: tdfwaitlist
--

CREATE TABLE public.alt_character (
    account_id bigint NOT NULL,
    alt_id bigint NOT NULL
);


ALTER TABLE public.alt_character OWNER TO tdfwaitlist;

--
-- Name: ban; Type: TABLE; Schema: public; Owner: tdfwaitlist
--

CREATE TABLE public.ban (
    kind character varying(11) NOT NULL,
    id bigint NOT NULL,
    expires_at timestamp without time zone,
    added_by bigint,
    reason text
);


ALTER TABLE public.ban OWNER TO tdfwaitlist;

--
-- Name: character; Type: TABLE; Schema: public; Owner: tdfwaitlist
--

CREATE TABLE public."character" (
    id bigint NOT NULL,
    name character varying(255) NOT NULL
);


ALTER TABLE public."character" OWNER TO tdfwaitlist;

--
-- Name: character_note; Type: TABLE; Schema: public; Owner: tdfwaitlist
--

CREATE TABLE public.character_note (
    id bigint NOT NULL,
    character_id bigint NOT NULL,
    author_id bigint NOT NULL,
    note text NOT NULL,
    logged_at bigint NOT NULL
);


ALTER TABLE public.character_note OWNER TO tdfwaitlist;

--
-- Name: character_note_id_seq; Type: SEQUENCE; Schema: public; Owner: tdfwaitlist
--

CREATE SEQUENCE public.character_note_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.character_note_id_seq OWNER TO tdfwaitlist;

--
-- Name: character_note_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: tdfwaitlist
--

ALTER SEQUENCE public.character_note_id_seq OWNED BY public.character_note.id;


--
-- Name: fit_history; Type: TABLE; Schema: public; Owner: tdfwaitlist
--

CREATE TABLE public.fit_history (
    id bigint NOT NULL,
    character_id bigint NOT NULL,
    fit_id bigint NOT NULL,
    implant_set_id bigint NOT NULL,
    logged_at bigint NOT NULL
);


ALTER TABLE public.fit_history OWNER TO tdfwaitlist;

--
-- Name: fit_history_id_seq; Type: SEQUENCE; Schema: public; Owner: tdfwaitlist
--

CREATE SEQUENCE public.fit_history_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.fit_history_id_seq OWNER TO tdfwaitlist;

--
-- Name: fit_history_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: tdfwaitlist
--

ALTER SEQUENCE public.fit_history_id_seq OWNED BY public.fit_history.id;


--
-- Name: fitting; Type: TABLE; Schema: public; Owner: tdfwaitlist
--

CREATE TABLE public.fitting (
    id bigint NOT NULL,
    dna character varying(1024) NOT NULL,
    hull integer NOT NULL
);


ALTER TABLE public.fitting OWNER TO tdfwaitlist;

--
-- Name: fitting_id_seq; Type: SEQUENCE; Schema: public; Owner: tdfwaitlist
--

CREATE SEQUENCE public.fitting_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.fitting_id_seq OWNER TO tdfwaitlist;

--
-- Name: fitting_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: tdfwaitlist
--

ALTER SEQUENCE public.fitting_id_seq OWNED BY public.fitting.id;


--
-- Name: fleet; Type: TABLE; Schema: public; Owner: tdfwaitlist
--

CREATE TABLE public.fleet (
    id bigint NOT NULL,
    boss_id bigint NOT NULL,
    is_updating boolean
);


ALTER TABLE public.fleet OWNER TO tdfwaitlist;

--
-- Name: fleet_activity; Type: TABLE; Schema: public; Owner: tdfwaitlist
--

CREATE TABLE public.fleet_activity (
    id bigint NOT NULL,
    character_id bigint NOT NULL,
    fleet_id bigint NOT NULL,
    first_seen bigint NOT NULL,
    last_seen bigint NOT NULL,
    hull integer NOT NULL,
    has_left boolean NOT NULL,
    is_boss boolean NOT NULL
);


ALTER TABLE public.fleet_activity OWNER TO tdfwaitlist;

--
-- Name: fleet_activity_id_seq; Type: SEQUENCE; Schema: public; Owner: tdfwaitlist
--

CREATE SEQUENCE public.fleet_activity_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.fleet_activity_id_seq OWNER TO tdfwaitlist;

--
-- Name: fleet_activity_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: tdfwaitlist
--

ALTER SEQUENCE public.fleet_activity_id_seq OWNED BY public.fleet_activity.id;


--
-- Name: fleet_squad; Type: TABLE; Schema: public; Owner: tdfwaitlist
--

CREATE TABLE public.fleet_squad (
    fleet_id bigint NOT NULL,
    category character varying(10) NOT NULL,
    wing_id bigint NOT NULL,
    squad_id bigint NOT NULL
);


ALTER TABLE public.fleet_squad OWNER TO tdfwaitlist;

--
-- Name: implant_set; Type: TABLE; Schema: public; Owner: tdfwaitlist
--

CREATE TABLE public.implant_set (
    id bigint NOT NULL,
    implants character varying(255) NOT NULL
);


ALTER TABLE public.implant_set OWNER TO tdfwaitlist;

--
-- Name: implant_set_id_seq; Type: SEQUENCE; Schema: public; Owner: tdfwaitlist
--

CREATE SEQUENCE public.implant_set_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.implant_set_id_seq OWNER TO tdfwaitlist;

--
-- Name: implant_set_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: tdfwaitlist
--

ALTER SEQUENCE public.implant_set_id_seq OWNED BY public.implant_set.id;


--
-- Name: refresh_token; Type: TABLE; Schema: public; Owner: tdfwaitlist
--

CREATE TABLE public.refresh_token (
    character_id bigint NOT NULL,
    refresh_token text NOT NULL,
    scopes character varying(1024) NOT NULL
);


ALTER TABLE public.refresh_token OWNER TO tdfwaitlist;

--
-- Name: skill_current; Type: TABLE; Schema: public; Owner: tdfwaitlist
--

CREATE TABLE public.skill_current (
    character_id bigint NOT NULL,
    skill_id integer NOT NULL,
    level smallint NOT NULL
);


ALTER TABLE public.skill_current OWNER TO tdfwaitlist;

--
-- Name: skill_history; Type: TABLE; Schema: public; Owner: tdfwaitlist
--

CREATE TABLE public.skill_history (
    id bigint NOT NULL,
    character_id bigint NOT NULL,
    skill_id integer NOT NULL,
    old_level smallint NOT NULL,
    new_level smallint NOT NULL,
    logged_at bigint NOT NULL
);


ALTER TABLE public.skill_history OWNER TO tdfwaitlist;

--
-- Name: skill_history_id_seq; Type: SEQUENCE; Schema: public; Owner: tdfwaitlist
--

CREATE SEQUENCE public.skill_history_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.skill_history_id_seq OWNER TO tdfwaitlist;

--
-- Name: skill_history_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: tdfwaitlist
--

ALTER SEQUENCE public.skill_history_id_seq OWNED BY public.skill_history.id;


--
-- Name: waitlist; Type: TABLE; Schema: public; Owner: tdfwaitlist
--

CREATE TABLE public.waitlist (
    id bigint NOT NULL,
    name character varying(255) NOT NULL,
    is_open boolean NOT NULL,
    is_archived boolean NOT NULL
);


ALTER TABLE public.waitlist OWNER TO tdfwaitlist;

--
-- Name: waitlist_entry; Type: TABLE; Schema: public; Owner: tdfwaitlist
--

CREATE TABLE public.waitlist_entry (
    id bigint NOT NULL,
    waitlist_id bigint NOT NULL,
    account_id bigint NOT NULL,
    joined_at bigint NOT NULL
);


ALTER TABLE public.waitlist_entry OWNER TO tdfwaitlist;

--
-- Name: waitlist_entry_fit; Type: TABLE; Schema: public; Owner: tdfwaitlist
--

CREATE TABLE public.waitlist_entry_fit (
    id bigint NOT NULL,
    character_id bigint NOT NULL,
    entry_id bigint NOT NULL,
    fit_id bigint NOT NULL,
    implant_set_id bigint NOT NULL,
    approved boolean NOT NULL,
    tags character varying(255) NOT NULL,
    category character varying(10) NOT NULL,
    fit_analysis text,
    review_comment text,
    cached_time_in_fleet bigint NOT NULL,
    is_alt boolean NOT NULL
);


ALTER TABLE public.waitlist_entry_fit OWNER TO tdfwaitlist;

--
-- Name: waitlist_entry_fit_id_seq; Type: SEQUENCE; Schema: public; Owner: tdfwaitlist
--

CREATE SEQUENCE public.waitlist_entry_fit_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.waitlist_entry_fit_id_seq OWNER TO tdfwaitlist;

--
-- Name: waitlist_entry_fit_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: tdfwaitlist
--

ALTER SEQUENCE public.waitlist_entry_fit_id_seq OWNED BY public.waitlist_entry_fit.id;


--
-- Name: waitlist_entry_id_seq; Type: SEQUENCE; Schema: public; Owner: tdfwaitlist
--

CREATE SEQUENCE public.waitlist_entry_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.waitlist_entry_id_seq OWNER TO tdfwaitlist;

--
-- Name: waitlist_entry_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: tdfwaitlist
--

ALTER SEQUENCE public.waitlist_entry_id_seq OWNED BY public.waitlist_entry.id;


--
-- Name: waitlist_id_seq; Type: SEQUENCE; Schema: public; Owner: tdfwaitlist
--

CREATE SEQUENCE public.waitlist_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.waitlist_id_seq OWNER TO tdfwaitlist;

--
-- Name: waitlist_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: tdfwaitlist
--

ALTER SEQUENCE public.waitlist_id_seq OWNED BY public.waitlist.id;


--
-- Name: character_note id; Type: DEFAULT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.character_note ALTER COLUMN id SET DEFAULT nextval('public.character_note_id_seq'::regclass);


--
-- Name: fit_history id; Type: DEFAULT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.fit_history ALTER COLUMN id SET DEFAULT nextval('public.fit_history_id_seq'::regclass);


--
-- Name: fitting id; Type: DEFAULT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.fitting ALTER COLUMN id SET DEFAULT nextval('public.fitting_id_seq'::regclass);


--
-- Name: fleet_activity id; Type: DEFAULT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.fleet_activity ALTER COLUMN id SET DEFAULT nextval('public.fleet_activity_id_seq'::regclass);


--
-- Name: implant_set id; Type: DEFAULT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.implant_set ALTER COLUMN id SET DEFAULT nextval('public.implant_set_id_seq'::regclass);


--
-- Name: skill_history id; Type: DEFAULT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.skill_history ALTER COLUMN id SET DEFAULT nextval('public.skill_history_id_seq'::regclass);


--
-- Name: waitlist id; Type: DEFAULT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.waitlist ALTER COLUMN id SET DEFAULT nextval('public.waitlist_id_seq'::regclass);


--
-- Name: waitlist_entry id; Type: DEFAULT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.waitlist_entry ALTER COLUMN id SET DEFAULT nextval('public.waitlist_entry_id_seq'::regclass);


--
-- Name: waitlist_entry_fit id; Type: DEFAULT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.waitlist_entry_fit ALTER COLUMN id SET DEFAULT nextval('public.waitlist_entry_fit_id_seq'::regclass);


--
-- Name: access_token_esi access_token_copy1_pkey; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.access_token_esi
    ADD CONSTRAINT access_token_copy1_pkey PRIMARY KEY (character_id);


--
-- Name: access_token access_token_pkey; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.access_token
    ADD CONSTRAINT access_token_pkey PRIMARY KEY (character_id);


--
-- Name: admins admins_pkey; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.admins
    ADD CONSTRAINT admins_pkey PRIMARY KEY (character_id);


--
-- Name: alt_character alt_character_pkey; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.alt_character
    ADD CONSTRAINT alt_character_pkey PRIMARY KEY (account_id, alt_id);


--
-- Name: ban ban_pkey; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.ban
    ADD CONSTRAINT ban_pkey PRIMARY KEY (kind, id);


--
-- Name: character_note character_note_pkey; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.character_note
    ADD CONSTRAINT character_note_pkey PRIMARY KEY (id);


--
-- Name: character character_pkey; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public."character"
    ADD CONSTRAINT character_pkey PRIMARY KEY (id);


--
-- Name: fitting dna; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.fitting
    ADD CONSTRAINT dna UNIQUE (dna);


--
-- Name: fit_history fit_history_pkey; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.fit_history
    ADD CONSTRAINT fit_history_pkey PRIMARY KEY (id);


--
-- Name: fitting fitting_pkey; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.fitting
    ADD CONSTRAINT fitting_pkey PRIMARY KEY (id);


--
-- Name: fleet_activity fleet_activity_pkey; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.fleet_activity
    ADD CONSTRAINT fleet_activity_pkey PRIMARY KEY (id);


--
-- Name: fleet fleet_pkey; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.fleet
    ADD CONSTRAINT fleet_pkey PRIMARY KEY (id);


--
-- Name: fleet_squad fleet_squad_pkey; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.fleet_squad
    ADD CONSTRAINT fleet_squad_pkey PRIMARY KEY (fleet_id, category);


--
-- Name: implant_set implant_set_pkey; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.implant_set
    ADD CONSTRAINT implant_set_pkey PRIMARY KEY (id);


--
-- Name: implant_set implants; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.implant_set
    ADD CONSTRAINT implants UNIQUE (implants);


--
-- Name: refresh_token refresh_token_pkey; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.refresh_token
    ADD CONSTRAINT refresh_token_pkey PRIMARY KEY (character_id);


--
-- Name: skill_current skill_current_pkey; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.skill_current
    ADD CONSTRAINT skill_current_pkey PRIMARY KEY (character_id, skill_id);


--
-- Name: skill_history skill_history_pkey; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.skill_history
    ADD CONSTRAINT skill_history_pkey PRIMARY KEY (id);


--
-- Name: waitlist_entry_fit waitlist_entry_fit_pkey; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.waitlist_entry_fit
    ADD CONSTRAINT waitlist_entry_fit_pkey PRIMARY KEY (id);


--
-- Name: waitlist_entry waitlist_entry_pkey; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.waitlist_entry
    ADD CONSTRAINT waitlist_entry_pkey PRIMARY KEY (id);


--
-- Name: waitlist_entry waitlist_id; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.waitlist_entry
    ADD CONSTRAINT waitlist_id UNIQUE (waitlist_id, account_id);


--
-- Name: waitlist waitlist_pkey; Type: CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.waitlist
    ADD CONSTRAINT waitlist_pkey PRIMARY KEY (id);


--
-- Name: idx_alt_character_alt_id; Type: INDEX; Schema: public; Owner: tdfwaitlist
--

CREATE INDEX idx_alt_character_alt_id ON public.alt_character USING btree (alt_id);


--
-- Name: idx_fit_history_character_id; Type: INDEX; Schema: public; Owner: tdfwaitlist
--

CREATE INDEX idx_fit_history_character_id ON public.fit_history USING btree (character_id);


--
-- Name: idx_fit_history_fit_id; Type: INDEX; Schema: public; Owner: tdfwaitlist
--

CREATE INDEX idx_fit_history_fit_id ON public.fit_history USING btree (fit_id);


--
-- Name: idx_fit_history_implant_set_id; Type: INDEX; Schema: public; Owner: tdfwaitlist
--

CREATE INDEX idx_fit_history_implant_set_id ON public.fit_history USING btree (implant_set_id);


--
-- Name: idx_fleet_boss_id; Type: INDEX; Schema: public; Owner: tdfwaitlist
--

CREATE INDEX idx_fleet_boss_id ON public.fleet USING btree (boss_id);


--
-- Name: idx_skill_history_character_id; Type: INDEX; Schema: public; Owner: tdfwaitlist
--

CREATE INDEX idx_skill_history_character_id ON public.skill_history USING btree (character_id);


--
-- Name: idx_waitlist_entry_account_id; Type: INDEX; Schema: public; Owner: tdfwaitlist
--

CREATE INDEX idx_waitlist_entry_account_id ON public.waitlist_entry USING btree (account_id);


--
-- Name: idx_waitlist_entry_fit_character_id; Type: INDEX; Schema: public; Owner: tdfwaitlist
--

CREATE INDEX idx_waitlist_entry_fit_character_id ON public.waitlist_entry_fit USING btree (character_id);


--
-- Name: idx_waitlist_entry_fit_entry_id; Type: INDEX; Schema: public; Owner: tdfwaitlist
--

CREATE INDEX idx_waitlist_entry_fit_entry_id ON public.waitlist_entry_fit USING btree (entry_id);


--
-- Name: idx_waitlist_entry_fit_fit_id; Type: INDEX; Schema: public; Owner: tdfwaitlist
--

CREATE INDEX idx_waitlist_entry_fit_fit_id ON public.waitlist_entry_fit USING btree (fit_id);


--
-- Name: idx_waitlist_entry_fit_implant_set_id; Type: INDEX; Schema: public; Owner: tdfwaitlist
--

CREATE INDEX idx_waitlist_entry_fit_implant_set_id ON public.waitlist_entry_fit USING btree (implant_set_id);


--
-- Name: access_token_esi access_token_copy1_character_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.access_token_esi
    ADD CONSTRAINT access_token_copy1_character_id_fkey FOREIGN KEY (character_id) REFERENCES public."character"(id);


--
-- Name: refresh_token access_token_ibfk_1; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.refresh_token
    ADD CONSTRAINT access_token_ibfk_1 FOREIGN KEY (character_id) REFERENCES public."character"(id);


--
-- Name: access_token access_token_ibfk_1; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.access_token
    ADD CONSTRAINT access_token_ibfk_1 FOREIGN KEY (character_id) REFERENCES public."character"(id);


--
-- Name: admins admins_ibfk_1; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.admins
    ADD CONSTRAINT admins_ibfk_1 FOREIGN KEY (character_id) REFERENCES public."character"(id);


--
-- Name: alt_character alt_character_ibfk_1; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.alt_character
    ADD CONSTRAINT alt_character_ibfk_1 FOREIGN KEY (account_id) REFERENCES public."character"(id);


--
-- Name: alt_character alt_character_ibfk_2; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.alt_character
    ADD CONSTRAINT alt_character_ibfk_2 FOREIGN KEY (alt_id) REFERENCES public."character"(id);


--
-- Name: ban ban_added_by_fk; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.ban
    ADD CONSTRAINT ban_added_by_fk FOREIGN KEY (added_by) REFERENCES public."character"(id);


--
-- Name: character_note character_note_ibfk_1; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.character_note
    ADD CONSTRAINT character_note_ibfk_1 FOREIGN KEY (character_id) REFERENCES public."character"(id);


--
-- Name: character_note character_note_ibfk_2; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.character_note
    ADD CONSTRAINT character_note_ibfk_2 FOREIGN KEY (author_id) REFERENCES public."character"(id);


--
-- Name: fit_history fit_history_ibfk_1; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.fit_history
    ADD CONSTRAINT fit_history_ibfk_1 FOREIGN KEY (character_id) REFERENCES public."character"(id);


--
-- Name: fit_history fit_history_ibfk_2; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.fit_history
    ADD CONSTRAINT fit_history_ibfk_2 FOREIGN KEY (fit_id) REFERENCES public.fitting(id);


--
-- Name: fit_history fit_history_ibfk_3; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.fit_history
    ADD CONSTRAINT fit_history_ibfk_3 FOREIGN KEY (implant_set_id) REFERENCES public.implant_set(id);


--
-- Name: fleet_activity fleet_activity_ibfk_1; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.fleet_activity
    ADD CONSTRAINT fleet_activity_ibfk_1 FOREIGN KEY (character_id) REFERENCES public."character"(id);


--
-- Name: fleet fleet_ibfk_1; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.fleet
    ADD CONSTRAINT fleet_ibfk_1 FOREIGN KEY (boss_id) REFERENCES public."character"(id);


--
-- Name: fleet_squad fleet_squad_ibfk_1; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.fleet_squad
    ADD CONSTRAINT fleet_squad_ibfk_1 FOREIGN KEY (fleet_id) REFERENCES public.fleet(id);


--
-- Name: skill_current skill_current_ibfk_1; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.skill_current
    ADD CONSTRAINT skill_current_ibfk_1 FOREIGN KEY (character_id) REFERENCES public."character"(id);


--
-- Name: skill_history skill_history_ibfk_1; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.skill_history
    ADD CONSTRAINT skill_history_ibfk_1 FOREIGN KEY (character_id) REFERENCES public."character"(id);


--
-- Name: waitlist_entry_fit waitlist_entry_fit_ibfk_1; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.waitlist_entry_fit
    ADD CONSTRAINT waitlist_entry_fit_ibfk_1 FOREIGN KEY (character_id) REFERENCES public."character"(id);


--
-- Name: waitlist_entry_fit waitlist_entry_fit_ibfk_2; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.waitlist_entry_fit
    ADD CONSTRAINT waitlist_entry_fit_ibfk_2 FOREIGN KEY (entry_id) REFERENCES public.waitlist_entry(id);


--
-- Name: waitlist_entry_fit waitlist_entry_fit_ibfk_3; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.waitlist_entry_fit
    ADD CONSTRAINT waitlist_entry_fit_ibfk_3 FOREIGN KEY (fit_id) REFERENCES public.fitting(id);


--
-- Name: waitlist_entry_fit waitlist_entry_fit_ibfk_4; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.waitlist_entry_fit
    ADD CONSTRAINT waitlist_entry_fit_ibfk_4 FOREIGN KEY (implant_set_id) REFERENCES public.implant_set(id);


--
-- Name: waitlist_entry waitlist_entry_ibfk_1; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.waitlist_entry
    ADD CONSTRAINT waitlist_entry_ibfk_1 FOREIGN KEY (waitlist_id) REFERENCES public.waitlist(id);


--
-- Name: waitlist_entry waitlist_entry_ibfk_2; Type: FK CONSTRAINT; Schema: public; Owner: tdfwaitlist
--

ALTER TABLE ONLY public.waitlist_entry
    ADD CONSTRAINT waitlist_entry_ibfk_2 FOREIGN KEY (account_id) REFERENCES public."character"(id);


--
-- PostgreSQL database dump complete
--

