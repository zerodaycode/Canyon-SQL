CREATE DATABASE public;

CREATE TABLE public.league (
    id					INT AUTO_INCREMENT PRIMARY KEY,
	ext_id				BIGINT NOT NULL,
    slug				TEXT NOT NULL,
	name				TEXT NOT NULL,
	region				TEXT NOT NULL,
	image_url			TEXT NOT NULL
);

CREATE TABLE public.tournament (
	id					INT AUTO_INCREMENT PRIMARY KEY,
	ext_id				BIGINT NOT NULL,
	slug				TEXT NOT NULL,
	start_date			DATE NOT NULL,
	end_date			DATE NOT NULL,
	league				INT,
	FOREIGN KEY (league) REFERENCES league(id)

);

CREATE TABLE public.player (
    id					INT AUTO_INCREMENT PRIMARY KEY,
	ext_id				BIGINT NOT NULL,
    first_name			TEXT NOT NULL,
	last_name			TEXT NOT NULL,
	summoner_name		TEXT NOT NULL,
	image_url			TEXT,
	role				TEXT NOT NULL
);

CREATE TABLE public.team (
    id					INT AUTO_INCREMENT PRIMARY KEY,
	ext_id				BIGINT NOT NULL,
    slug				TEXT NOT NULL,
	name				TEXT NOT NULL,
	code				TEXT NOT NULL,
	image_url			TEXT NOT NULL,
	alt_image_url		TEXT,
	bg_image_url		TEXT,
	home_league			INT,
	FOREIGN KEY (home_league) REFERENCES league(id)
);
