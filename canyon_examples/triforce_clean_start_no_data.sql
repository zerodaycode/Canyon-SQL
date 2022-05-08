DROP database IF EXISTS triforce;

DROP USER IF EXISTS triforce;

CREATE ROLE triforce WITH LOGIN SUPERUSER PASSWORD 'abc123..';

CREATE DATABASE triforce WITH TEMPLATE = template0 ENCODING = 'UTF8' LOCALE = 'es_ES.UTF-8';

ALTER DATABASE triforce OWNER TO triforce;

\c triforce

CREATE TABLE public.league (
     id					INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	 ext_id				BIGINT NOT NULL,
     slug				TEXT NOT NULL,
	 name				TEXT NOT NULL,
	 region				TEXT NOT NULL,
	 image_url			TEXT NOT NULL
);
CREATE TABLE public.tournament (
	id					INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	ext_id				BIGINT NOT NULL,
	slug				TEXT NOT NULL,
	start_date			DATE NOT NULL,
	end_date			DATE NOT NULL,
	league				INTEGER REFERENCES league(id)
);

CREATE TABLE public.player (
     id					INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	 ext_id				BIGINT NOT NULL,
     first_name			TEXT NOT NULL,
	 last_name			TEXT NOT NULL,
	 summoner_name		TEXT NOT NULL,
	 image_url			TEXT,
	 role				TEXT NOT NULL
);

CREATE TABLE public.team (
     id					INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	 ext_id				BIGINT NOT NULL,
     slug				TEXT NOT NULL,
	 name				TEXT NOT NULL,
	 code				TEXT NOT NULL,
	 image_url			TEXT NOT NULL,
	 alt_image_url		TEXT,
	 bg_image_url		TEXT,
	 home_league		INTEGER REFERENCES league(id)
);

CREATE TABLE public.team_player (
     id					INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	 team_id			INTEGER REFERENCES team(id) ON DELETE CASCADE,
     player_id			INTEGER REFERENCES player(id) ON DELETE CASCADE
);

ALTER TABLE public.league OWNER TO triforce;
ALTER TABLE public.tournament OWNER TO triforce;
ALTER TABLE public.player OWNER TO triforce;
ALTER TABLE public.team OWNER TO triforce;
ALTER TABLE public.team_player OWNER TO triforce;
