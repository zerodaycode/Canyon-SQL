///! Constant values to share accross the integration tests
pub const PSQL_DS: &str = "postgres_docker";
pub const SQL_SERVER_DS: &str = "sqlserver_docker";

pub static FETCH_PUBLIC_SCHEMA: &str =
"SELECT
    gi.table_name,
    gi.column_name,
    gi.data_type,
    gi.character_maximum_length,
    gi.is_nullable,
    gi.column_default,
    gi.numeric_precision,
    gi.numeric_scale,
    gi.numeric_precision_radix,
    gi.datetime_precision,
    gi.interval_type,
    CASE WHEN starts_with(CAST(pg_catalog.pg_get_constraintdef(oid) AS TEXT), 'FOREIGN KEY')
        THEN CAST(pg_catalog.pg_get_constraintdef(oid) AS TEXT) ELSE NULL END AS foreign_key_info,
    CASE WHEN starts_with(CAST(pg_catalog.pg_get_constraintdef(oid) AS TEXT), 'FOREIGN KEY')
        THEN con.conname ELSE NULL END AS foreign_key_name,
    CASE WHEN starts_with(CAST(pg_catalog.pg_get_constraintdef(oid) AS TEXT), 'PRIMARY KEY')
        THEN CAST(pg_catalog.pg_get_constraintdef(oid) AS TEXT) ELSE NULL END AS primary_key_info,
    CASE WHEN starts_with(CAST(pg_catalog.pg_get_constraintdef(oid) AS TEXT), 'PRIMARY KEY')
        THEN con.conname ELSE NULL END AS primary_key_name,
    gi.is_identity,
    gi.identity_generation
FROM
    information_schema.columns AS gi
LEFT JOIN pg_catalog.pg_constraint AS con on
    gi.table_name = CAST(con.conrelid::regclass AS TEXT) AND
    gi.column_name = split_part(split_part(CAST(pg_catalog.pg_get_constraintdef(oid) AS TEXT),')',1),'(',2)
WHERE
    table_schema = 'public';";


pub const SQL_SERVER_CREATE_TABLES: &str = "
IF OBJECT_ID(N'[dbo].[league]', N'U') IS NULL
BEGIN
    CREATE TABLE dbo.league (
        id					INT PRIMARY KEY IDENTITY,
        ext_id				BIGINT NOT NULL,
        slug				NVARCHAR(250) NOT NULL,
        name				NVARCHAR(250) NOT NULL,
        region				NVARCHAR(250) NOT NULL,
        image_url			NVARCHAR(250) NOT NULL
    );
END;

IF OBJECT_ID(N'[dbo].[tournament]', N'U') IS NULL
BEGIN
    CREATE TABLE dbo.tournament (
        id					INT PRIMARY KEY IDENTITY,
        ext_id				BIGINT NOT NULL,
        slug				NVARCHAR(250) NOT NULL,
        start_date			DATE NOT NULL,
        end_date			DATE NOT NULL,
        league				INT REFERENCES league(id)
    );
END;

IF OBJECT_ID(N'[dbo].[player]', N'U') IS NULL
BEGIN
    CREATE TABLE dbo.player (
        id					INT PRIMARY KEY IDENTITY,
        ext_id				BIGINT NOT NULL,
        first_name			NVARCHAR(250) NOT NULL,
        last_name			NVARCHAR(250) NOT NULL,
        summoner_name		NVARCHAR(250) NOT NULL,
        image_url			NVARCHAR(250),
        role				NVARCHAR(250) NOT NULL
    );
END;

IF OBJECT_ID(N'[dbo].[team]', N'U') IS NULL
BEGIN
    CREATE TABLE dbo.team (
        id					INT PRIMARY KEY IDENTITY,
        ext_id				BIGINT NOT NULL,
        slug				NVARCHAR(250) NOT NULL,
        name				NVARCHAR(250) NOT NULL,
        code				NVARCHAR(250) NOT NULL,
        image_url			NVARCHAR(250) NOT NULL,
        alt_image_url		NVARCHAR(250),
        bg_image_url		NVARCHAR(250),
        home_league		    INT REFERENCES league(id)
    );
END;
";

pub const SQL_SERVER_FILL_TABLE_VALUES: &str = "
-- Values for league table
-- Values for league table
SET IDENTITY_INSERT dbo.league ON;
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (1, 100695891328981122, 'european-masters', 'European Masters', 'EUROPE', 'http://static.lolesports.com/leagues/EM_Bug_Outline1.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (2, 101097443346691685, 'turkey-academy-league', 'TAL', 'TURKEY', 'http://static.lolesports.com/leagues/1592516072459_TAL-01-FullonDark.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (3, 101382741235120470, 'lla', 'LLA', 'LATIN AMERICA', 'http://static.lolesports.com/leagues/1592516315279_LLA-01-FullonDark.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (4, 104366947889790212, 'pcs', 'PCS', 'HONG KONG, MACAU, TAIWAN', 'http://static.lolesports.com/leagues/1592515942679_PCS-01-FullonDark.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (5, 105266074488398661, 'superliga', 'SuperLiga', 'EUROPE', 'http://static.lolesports.com/leagues/SL21-V-white.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (6, 105266088231437431, 'ultraliga', 'Ultraliga', 'EUROPE', 'http://static.lolesports.com/leagues/1639390623717_ULTRALIGA_logo_sq_cyan.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (7, 105266091639104326, 'primeleague', 'Prime League', 'EUROPE', 'http://static.lolesports.com/leagues/PrimeLeagueResized.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (8, 105266094998946936, 'pg_nationals', 'PG Nationals', 'EUROPE', 'http://static.lolesports.com/leagues/PG_Nationals_Logo_White.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (9, 105266098308571975, 'nlc', 'NLC', 'EUROPE', 'http://static.lolesports.com/leagues/1641490922073_nlc_logo.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (10, 105266101075764040, 'liga_portuguesa', 'Liga Portuguesa', 'EUROPE', 'http://static.lolesports.com/leagues/1649884876085_LPLOL_2021_ISO_G-c389e9ae85c243e4f76a8028bbd9ca1609c2d12bc47c3709a9250d1b3ca43f58.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (11, 105266103462388553, 'lfl', 'La Ligue Française', 'EUROPE', 'http://static.lolesports.com/leagues/LFL_Logo_2020_black1.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (12, 105266106309666619, 'hitpoint_masters', 'Hitpoint Masters', 'EUROPE', 'http://static.lolesports.com/leagues/1641465237186_HM_white.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (13, 105266108767593290, 'greek_legends', 'Greek Legends League', 'EUROPE', 'http://static.lolesports.com/leagues/GLL_LOGO_WHITE.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (14, 105266111679554379, 'esports_balkan_league', 'Esports Balkan League', 'EUROPE', 'http://static.lolesports.com/leagues/1625735031226_ebl_crest-whitePNG.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (15, 105549980953490846, 'cblol_academy', 'CBLOL Academy', 'BRAZIL', 'http://static.lolesports.com/leagues/cblol-acad-white.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (16, 105709090213554609, 'lco', 'LCO', 'OCEANIA', 'http://static.lolesports.com/leagues/lco-color-white.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (17, 106827757669296909, 'ljl_academy', 'LJL Academy', 'JAPAN', 'http://static.lolesports.com/leagues/1630062215891_ljl-al_logo_gradient.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (18, 107213827295848783, 'vcs', 'VCS', 'VIETNAM', 'http://static.lolesports.com/leagues/1635953171501_LOL_VCS_Full_White.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (19, 107407335299756365, 'elite_series', 'Elite Series', 'EUROPE', 'http://static.lolesports.com/leagues/1641287979138_EliteSeriesMarkWhite.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (20, 107581050201097472, 'honor_division', 'Honor Division', 'LATIN AMERICA', 'http://static.lolesports.com/leagues/1641750781829_divhonormxwhite.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (21, 107581669166925444, 'elements_league', 'Elements League', 'LATIN AMERICA', 'http://static.lolesports.com/leagues/1642593573670_LOGO_ELEMENTS_White.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (22, 107582133359724496, 'volcano_discover_league', 'Volcano League', 'LATIN AMERICA', 'http://static.lolesports.com/leagues/1643106609661_VOLCANO-VERTICAL-ColorLight.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (23, 107582580502415838, 'claro_gaming_stars_league', 'Stars League', 'LATIN AMERICA', 'http://static.lolesports.com/leagues/1642595169468_CLARO-GAMING-STARS-LEAGUE-B.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (24, 107598636564896416, 'master_flow_league', 'Master Flow League', 'LATIN AMERICA', 'http://static.lolesports.com/leagues/1643794656405_LMF-White.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (25, 107598951349015984, 'honor_league', 'Honor League', 'LATIN AMERICA', 'http://static.lolesports.com/leagues/1643036660690_lhe-ColorLight.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (26, 107603541524308819, 'movistar_fiber_golden_league', 'Golden League', 'LATIN AMERICA', 'http://static.lolesports.com/leagues/1642445572375_MovistarLeague.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (27, 107898214974993351, 'college_championship', 'College Championship', 'NORTH AMERICA', 'http://static.lolesports.com/leagues/1646396098648_CollegeChampionshiplogo.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (28, 107921249454961575, 'proving_grounds', 'Proving Grounds', 'NORTH AMERICA', 'http://static.lolesports.com/leagues/1646747578708_download8.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (29, 108001239847565215, 'tft_esports', 'TFT Last Chance Qualifier', 'INTERNATIONAL', 'http://static.lolesports.com/leagues/1649439858579_tftesport.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (30, 98767975604431411, 'worlds', 'Worlds', 'INTERNATIONAL', 'http://static.lolesports.com/leagues/1592594612171_WorldsDarkBG.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (31, 98767991295297326, 'all-star', 'All-Star Event', 'INTERNATIONAL', 'http://static.lolesports.com/leagues/1592594737227_ASEDarkBG.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (32, 98767991299243165, 'lcs', 'LCS', 'NORTH AMERICA', 'http://static.lolesports.com/leagues/LCSNew-01-FullonDark.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (33, 98767991302996019, 'lec', 'LEC', 'EUROPE', 'http://static.lolesports.com/leagues/1592516184297_LEC-01-FullonDark.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (34, 98767991310872058, 'lck', 'LCK', 'KOREA', 'http://static.lolesports.com/leagues/lck-color-on-black.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (35, 98767991314006698, 'lpl', 'LPL', 'CHINA', 'http://static.lolesports.com/leagues/1592516115322_LPL-01-FullonDark.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (36, 98767991325878492, 'msi', 'MSI', 'INTERNATIONAL', 'http://static.lolesports.com/leagues/1592594634248_MSIDarkBG.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (37, 98767991332355509, 'cblol-brazil', 'CBLOL', 'BRAZIL', 'http://static.lolesports.com/leagues/cblol-logo-symbol-offwhite.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (38, 98767991335774713, 'lck_challengers_league', 'LCK Challengers', 'KOREA', 'http://static.lolesports.com/leagues/lck-cl-white.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (39, 98767991343597634, 'turkiye-sampiyonluk-ligi', 'TCL', 'TURKEY', 'https://lolstatic-a.akamaihd.net/esports-assets/production/league/turkiye-sampiyonluk-ligi-8r9ofb9.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (40, 98767991349978712, 'ljl-japan', 'LJL', 'JAPAN', 'http://static.lolesports.com/leagues/1592516354053_LJL-01-FullonDark.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (41, 98767991355908944, 'lcl', 'LCL', 'COMMONWEALTH OF INDEPENDENT STATES', 'http://static.lolesports.com/leagues/1593016885758_LCL-01-FullonDark.png');
INSERT INTO dbo.league (id,ext_id,slug,name,region,image_url) VALUES (42, 99332500638116286, 'lcs-academy', 'LCS Academy', 'NORTH AMERICA', 'http://static.lolesports.com/leagues/lcs-academy-purple.png');
SET IDENTITY_INSERT dbo.league OFF;

-- Values for player table
SET IDENTITY_INSERT dbo.player ON;
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (1, 98767975906852059, 'Jaehyeok', 'Park', 'Ruler', 'http://static.lolesports.com/players/1642153903692_GEN_Ruler_F.png', 'bottom');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (2, 102186485482484390, 'Hyeonjun', 'Choi', 'Doran', 'http://static.lolesports.com/players/1642153880932_GEN_Doran_F.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (3, 98767975916458257, 'Wangho ', 'Han', 'Peanut', 'http://static.lolesports.com/players/1642153896918_GEN_peanut_A.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (4, 99871276342168416, 'Jihun', 'Jung', 'Chovy', 'http://static.lolesports.com/players/1642153873969_GEN_Chovy_F.png', 'mid');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (5, 99871276332909841, 'Siu', 'Son', 'Lehends', 'http://static.lolesports.com/players/1642153887731_GEN_Lehends_F.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (6, 104266797862156067, 'Youngjae', 'Ko', 'YoungJae', 'http://static.lolesports.com/players/1642153913037_GEN_YoungJae_F.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (7, 103495716560217968, 'Hyoseong', 'Oh', 'Vsta', 'http://static.lolesports.com/players/1642154102606_HLE_Vsta_F.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (8, 104266795407626462, 'Dongju', 'Lee', 'DuDu', 'http://static.lolesports.com/players/1642154060441_HLE_DuDu_F.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (9, 106267386230851795, 'Junghyeun', 'Kim', 'Willer', 'http://static.lolesports.com/players/1642154110676_HLE_Willer_F.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (10, 100725844995692264, 'Janggyeom', 'Kim', 'OnFleek', 'http://static.lolesports.com/players/1642154084709_HLE_Onfleek_F.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (11, 105320683858945274, 'Hongjo', 'Kim', 'Karis', 'http://static.lolesports.com/players/1642154066010_HLE_Karis_F.png', 'mid');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (12, 104287359934240404, 'Jaehoon', 'Lee', 'SamD', 'http://static.lolesports.com/players/1642154094651_HLE_SamD_F.png', 'bottom');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (13, 103461966870841210, 'Wyllian', 'Adriano', 'asta', 'http://static.lolesports.com/players/1643226025146_Astacopy.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (14, 107559111166843860, 'Felipe', 'Boal', 'Boal', 'http://static.lolesports.com/players/1644095483228_BOALcopiar.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (15, 107559255871511679, 'Giovani', 'Baldan', 'Mito', 'http://static.lolesports.com/players/1643226193262_Mitocopy.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (16, 103478281329357326, 'Arthur', 'Machado', 'Tutsz', 'http://static.lolesports.com/players/1643226293749_Tutszcopy.png', 'mid');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (17, 103743599797538329, 'Luiz Felipe', 'Lobo', 'Flare', 'http://static.lolesports.com/players/1643226082718_Flarecopy.png', 'bottom');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (18, 99566408210057665, 'Natan', 'Braz', 'fNb', 'http://static.lolesports.com/players/1643226467130_Fnbcopiar.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (19, 99566407771166805, 'Filipe', 'Brombilla', 'Ranger', 'http://static.lolesports.com/players/1643226495379_Rangercopiar.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (20, 107559327426244686, 'Vinícius', 'Corrêa', 'StineR', 'http://static.lolesports.com/players/1643226666563_Silhueta.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (21, 99566407784212776, 'Bruno', 'Farias', 'Envy', 'http://static.lolesports.com/players/1643226430923_Envycopiar.png', 'mid');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (22, 107559338252333149, 'Gabriel', 'Furuuti', 'Fuuu', 'http://static.lolesports.com/players/1643226717192_Silhueta.png', 'mid');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (23, 105397181199735591, 'Lucas', 'Fensterseifer', 'Netuno', 'http://static.lolesports.com/players/1644095521735_Netunocopiar.png', 'bottom');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (24, 98767975947296513, 'Ygor', 'Freitas', 'RedBert', 'http://static.lolesports.com/players/1643226527904_Redbertcopiar.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (25, 100754278890207800, 'Geonyeong', 'Mun', 'Steal', 'http://static.lolesports.com/players/1644905307225_dfm_steal.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (26, 99566404536983507, 'Chanju', 'Lee', 'Yaharong', 'http://static.lolesports.com/players/1644905328869_dfm_yaharong.png', 'mid');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (27, 104016425624023728, 'Jiyoong', 'Lee', 'Harp', 'http://static.lolesports.com/players/1644905257358_dfm_harp.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (28, 98767991750309549, 'Danil', 'Reshetnikov', 'Diamondprox', 'http://static.lolesports.com/players/Diamondproxcopy.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (29, 105700748891875072, 'Nikita ', 'Gudkov', 'Griffon ', 'http://static.lolesports.com/players/1642071116433_placeholder.png', 'mid');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (30, 105700946934214905, 'YEVHEN', 'ZAVALNYI', 'Mytant', 'http://static.lolesports.com/players/1642071138150_placeholder.png', 'bottom');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (31, 98767991755955790, 'Eduard', 'Abgaryan', 'Edward', 'https://lolstatic-a.akamaihd.net/esports-assets/production/player/gosu-pepper-88anxcql.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (32, 106301600611225723, 'Mark', 'Leksin', 'Dreampull', 'http://static.lolesports.com/players/placeholder.jpg', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (33, 107721938219680332, 'Azamat', 'Atkanov', 'TESLA', 'http://static.lolesports.com/players/1643706327509_placeholder.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (34, 100725844988653773, 'Su', 'Heo', 'ShowMaker', 'http://static.lolesports.com/players/1642153659258_DK_ShowMaker_F.png', 'mid');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (35, 102483272156027229, 'Daegil', 'Seo', 'deokdam', 'http://static.lolesports.com/players/1642153629340_DK_deokdam_F.png', 'bottom');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (36, 101388913291808185, 'Hyeonggyu', 'Kim', 'Kellin', 'http://static.lolesports.com/players/1642153649009_DK_Kellin_F.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (37, 105705431649727017, 'Taeyoon', 'Noh', 'Burdol', 'http://static.lolesports.com/players/1642153598672_DK_Burdol_F.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (38, 103729432252832975, 'Yongho', 'Yoon', 'Hoya', 'http://static.lolesports.com/players/1642153639500_DK_Hoya_F.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (39, 105320703008048707, 'Dongbum', 'Kim', 'Croco', 'http://static.lolesports.com/players/1642154712531_LSB_Croco_R.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (40, 105501829364113001, 'Hobin', 'Jeon', 'Howling', 'http://static.lolesports.com/players/1642154731703_LSB_Howling_F.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (41, 104284310661848687, 'Juhyeon', 'Lee', 'Clozer', 'http://static.lolesports.com/players/1642154706000_LSB_Clozer_R.png', 'mid');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (42, 100725844996918206, 'Jaeyeon', 'Kim', 'Dove', 'http://static.lolesports.com/players/1642154719503_LSB_Dove_R.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (43, 105530583598805234, 'Myeongjun', 'Lee', 'Envyy', 'http://static.lolesports.com/players/1642154726047_LSB_Envyy_F.png', 'bottom');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (44, 105530584812980593, 'Jinhong', 'Kim', 'Kael', 'http://static.lolesports.com/players/1642154745002_LSB_Kael_F.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (45, 105501834624360050, 'Sanghoon', 'Yoon', 'Ice', 'http://static.lolesports.com/players/1642154738262_LSB_Ice_F.png', 'bottom');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (46, 99322214647978964, 'Daniele', 'di Mauro', 'Jiizuke', 'http://static.lolesports.com/players/eg-jiizuke-2021.png', 'mid');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (47, 100787602257283436, 'Minh Loc', 'Pham', 'Zeros', 'https://lolstatic-a.akamaihd.net/esports-assets/production/player/zeros-4keddu17.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (48, 104327502738107767, 'Nicolás', 'Rivero', 'Kiefer', 'http://static.lolesports.com/players/1643047365591_Kiefer-2.png', 'mid');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (49, 102179902322952953, 'Manuel', 'Scala', 'Pancake', 'http://static.lolesports.com/players/1643047550782_Pancake-5.png', 'bottom');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (50, 105516185566739968, 'Cristóbal', 'Arróspide', 'Zothve', 'http://static.lolesports.com/players/1643047287141_Zothve-9.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (51, 99871352196477603, 'Gwanghyeop', 'Kim', 'Hoglet', 'http://static.lolesports.com/players/1643047312405_Hoglet-8.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (52, 99871352193690418, 'Changhun', 'Han', 'Luci', 'http://static.lolesports.com/players/1643047438703_Luci-5.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (53, 107635899693202699, 'Thomas', 'Garnsworthy', 'Tronthepom', 'https://static.lolesports.com/players/download.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (54, 107635905118503535, 'James', 'Craig', 'Voice', 'https://static.lolesports.com/players/download.png', 'bottom');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (55, 107635907168238086, 'Rocco', 'Potter', 'rocco521', 'https://static.lolesports.com/players/download.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (56, 107635918452357647, 'Reuben', 'Best', 'Reufury', 'https://static.lolesports.com/players/download.png', 'mid');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (57, 107647480732814180, 'Bryce', 'Zhou', 'Meifan', 'https://static.lolesports.com/players/download.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (58, 107657801460158111, 'Benny', 'Nguyen', 'District 1', 'https://static.lolesports.com/players/download.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (59, 105709372540742118, 'Blake', 'Schlage', 'Azus', 'http://static.lolesports.com/players/silhouette.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (60, 106350759376304634, 'Shao', 'Zhong', 'Akano', 'https://static.lolesports.com/players/download.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (61, 107634941727734818, 'Jeremy', 'Lim', 'foreigner', 'https://static.lolesports.com/players/download.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (62, 105709381466108761, 'Reuben', 'Salb', 'Piglet', 'http://static.lolesports.com/players/silhouette.png', 'bottom');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (63, 105747861836427633, 'Yi', 'Chen', 'Thomas Shen', 'https://static.lolesports.com/players/download.png', 'bottom');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (64, 107657786356796634, 'Robert', 'Wells', 'Tyran', 'https://static.lolesports.com/players/download.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (65, 107657790493529410, 'Da Woon', 'Jeung', 'DaJeung', 'https://static.lolesports.com/players/download.png', 'mid');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (66, 107657793079479518, 'Rhett', 'Wiggins', 'Vxpir', 'https://static.lolesports.com/players/download.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (67, 107698225510856278, 'Benson', 'Tsai', 'Entrust', 'https://static.lolesports.com/players/download.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (68, 103525219435043049, 'Lachlan', 'Keene-O''Keefe', 'N0body', 'https://lolstatic-a.akamaihd.net/esports-assets/production/player/n0body-einjqvyk.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (69, 101389749294612370, 'Janik', 'Bartels', 'Jenax', 'http://static.lolesports.com/players/1642003381408_jenax.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (70, 101383793865143549, 'Erik', 'Wessén', 'Treatz', 'http://static.lolesports.com/players/1642003495533_treatz.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (71, 101389737455173027, 'Daniyal ', 'Gamani', 'Sertuss', 'http://static.lolesports.com/players/1642003453914_sertuss.png', 'mid');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (72, 99322214588927915, 'Erberk ', 'Demir', 'Gilius', 'http://static.lolesports.com/players/1642003341615_gilius.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (73, 99322214668103078, 'Matti', 'Sormunen', 'WhiteKnight', 'http://static.lolesports.com/players/1642003243059_white-knight.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (74, 100312190807221865, 'Nikolay ', 'Akatov', 'Zanzarah', 'http://static.lolesports.com/players/1642003282324_zanzarah.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (75, 99322214243134013, 'Hampus ', 'Abrahamsson', 'promisq', 'http://static.lolesports.com/players/1642003205916_promisq.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (76, 99322214620375780, 'Kasper', 'Kobberup', 'Kobbe', 'http://static.lolesports.com/players/1642003168563_kobbe.png', 'bottom');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (77, 99322214238585389, 'Patrik', 'Jiru', 'Patrik', 'http://static.lolesports.com/players/1642004060212_patrik.png', 'bottom');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (78, 105519722481834694, 'Mark', 'van Woensel', 'Markoon', 'http://static.lolesports.com/players/1642003998089_markoon.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (79, 105519724699493915, 'Hendrik', 'Reijenga', 'Advienne', 'http://static.lolesports.com/players/1642003935782_advienne.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (80, 99322214616775017, 'Erlend', 'Holm', 'Nukeduck', 'http://static.lolesports.com/players/1642004031937_nukeduck.png', 'mid');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (81, 101389713973624205, 'Finn', 'Wiestål', 'Finn', 'http://static.lolesports.com/players/1642003970167_finn.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (82, 99322214629661297, 'Mihael', 'Mehle', 'Mikyx', 'http://static.lolesports.com/players/G2_MIKYX2021_summer.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (83, 100482247959137902, 'Emil', 'Larsson', 'Larssen', 'http://static.lolesports.com/players/1642003206398_larssen.png', 'mid');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (84, 99322214598412197, 'Andrei', 'Pascu', 'Odoamne', 'http://static.lolesports.com/players/1642003264169_odoamne.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (85, 102181528883745160, 'Adrian', 'Trybus', 'Trymbi', 'http://static.lolesports.com/players/1642003301461_trymbi.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (86, 99566406053904433, 'Geun-seong', 'Kim', 'Malrang', 'http://static.lolesports.com/players/1642003233110_malrang.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (87, 103536921420956640, 'Markos', 'Stamkopoulos', 'Comp', 'http://static.lolesports.com/players/1642003175488_comp.png', 'bottom');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (88, 101388912808637770, 'Hanxi', 'Xia', 'Chelizi', 'http://static.lolesports.com/players/1593128001829_silhouette.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (89, 105516474039500339, 'Fei-Yang', 'Luo', 'Captain', 'http://static.lolesports.com/players/silhouette.png', 'mid');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (90, 106368709696011395, 'Seung Min', 'Han', 'Patch', 'http://static.lolesports.com/players/silhouette.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (91, 107597376599119596, 'HAOTIAN', 'BI', 'yaoyao', 'http://static.lolesports.com/players/1641805668544_placeholder.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (92, 101388912811586896, 'Zhilin', 'Su', 'Southwind', 'http://static.lolesports.com/players/1593129903866_ig-southwind-web.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (93, 101388912810603854, 'Wang', 'Ding', 'Puff', 'http://static.lolesports.com/players/1593129891452_ig-puff-web.png', 'bottom');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (94, 104287371427354335, 'Zhi-Peng', 'Tian', 'New', 'http://static.lolesports.com/players/1593132511529_rng-new-web.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (95, 107597380474228562, 'WANG', 'XIN', 'frigid', 'http://static.lolesports.com/players/1641805726386_placeholder.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (96, 104287365097341858, 'Peng', 'Guo', 'ppgod', 'http://static.lolesports.com/players/1593135580022_v5-ppgod-web.png', 'support');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (97, 103478281359738222, 'Qi-Shen ', 'Ying', 'Photic', 'https://lolstatic-a.akamaihd.net/esports-assets/production/player/photic-k1ttlyxh.png', 'bottom');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (98, 103478281402167891, 'Xiao-Long ', 'Li', 'XLB', 'http://static.lolesports.com/players/1593132528126_rng-xlb-web.png', 'jungle');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (99, 102186438403674539, 'Jaewon', 'Lee', 'Rich', 'http://static.lolesports.com/players/ns-rich.png', 'top');
INSERT INTO dbo.player (id,ext_id, first_name, last_name, summoner_name, image_url, role) VALUES (100, 99124844346233375, 'Onur', 'Ünalan', 'Zergsting', 'http://static.lolesports.com/players/1633542837856_gs-zergsting-w21.png', 'support');
SET IDENTITY_INSERT dbo.player OFF;

-- Values for team table
SET IDENTITY_INSERT dbo.team ON;
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (1, 100205573495116443, 'geng', 'Gen.G', 'GEN', 'http://static.lolesports.com/teams/1631819490111_geng-2021-worlds.png', 'http://static.lolesports.com/teams/1592589327624_Gen.GGEN-03-FullonLight.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/geng-bnm75bf5.png', 34);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (2, 100205573496804586, 'hanwha-life-esports', 'Hanwha Life Esports', 'HLE', 'http://static.lolesports.com/teams/1631819564399_hle-2021-worlds.png', 'http://static.lolesports.com/teams/hle-2021-color-on-light2.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/hanwha-life-esports-7kh5kjdc.png', 34);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (3, 100205576307813373, 'flamengo-esports', 'Flamengo Esports', 'FLA', 'http://static.lolesports.com/teams/1642953977323_Monograma_Branco-Vermelho.png', 'http://static.lolesports.com/teams/1642953977326_Monograma_Branco-Vermelho.png', NULL, 37);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (4, 100205576309502431, 'furia', 'FURIA', 'FUR', 'http://static.lolesports.com/teams/FURIA---black.png', 'http://static.lolesports.com/teams/FURIA---black.png', 'http://static.lolesports.com/teams/FuriaUppercutFUR.png', 37);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (5, 100285330168091787, 'detonation-focusme', 'DetonatioN FocusMe', 'DFM', 'http://static.lolesports.com/teams/1631820630246_dfm-2021-worlds.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/detonation-focusme-ajvyc8cy.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/detonation-focusme-4pgp383l.png', 40);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (6, 100289931264192378, 'team-spirit', 'Team Spirit', 'TSPT', 'http://static.lolesports.com/teams/1643720491696_Whitelogo.png', 'http://static.lolesports.com/teams/1643720491697_Blacklogo.png', NULL, 41);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (7, 100725845018863243, 'dwg-kia', 'DWG KIA', 'DK', 'http://static.lolesports.com/teams/1631819456274_dwg-kia-2021-worlds.png', 'http://static.lolesports.com/teams/DK-FullonLight.png', 'http://static.lolesports.com/teams/DamwonGamingDWG.png', 34);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (8, 100725845022060229, 'liiv-sandbox', 'Liiv SANDBOX', 'LSB', 'http://static.lolesports.com/teams/liiv-sandbox-new.png', 'http://static.lolesports.com/teams/liiv-sandbox-new.png', NULL, 34);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (9, 101157821444002947, 'nexus-blitz-pro-a', 'Nexus Blitz Blue', 'NXB', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/nexus-blitz-pro-a-esrcx58b.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/nexus-blitz-pro-a-3w3j1cwx.png', NULL, 31);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (10, 101157821447017610, 'nexus-blitz-pro-b', 'Nexus Blitz Red', 'NXR', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/nexus-blitz-pro-b-j6s80wmi.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/nexus-blitz-pro-b-kjtp467.png', NULL, 31);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (11, 101383792559569368, 'all-knights', 'All Knights', 'AK', 'http://static.lolesports.com/teams/AK-Black-BG.png', 'http://static.lolesports.com/teams/AK-White-BG.png', NULL, 3);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (12, 101383792887446028, 'mammoth', 'MAMMOTH', 'MEC', 'http://static.lolesports.com/teams/1643079304055_RedMammothIcon.png', 'http://static.lolesports.com/teams/1643079304062_RedMammothIcon.png', NULL, 16);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (13, 101383792891050518, 'gravitas', 'Gravitas', 'GRV', 'http://static.lolesports.com/teams/gravitas-logo.png', 'http://static.lolesports.com/teams/gravitas-logo.png', NULL, 16);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (14, 101383793567806688, 'sk-gaming', 'SK Gaming', 'SK', 'http://static.lolesports.com/teams/1643979272144_SK_Monochrome.png', 'http://static.lolesports.com/teams/1643979272151_SK_Monochrome.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/sk-gaming-2cd63tzz.png', 33);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (15, 101383793569248484, 'astralis', 'Astralis', 'AST', 'http://static.lolesports.com/teams/AST-FullonDark.png', 'http://static.lolesports.com/teams/AST-FullonLight.png', 'http://static.lolesports.com/teams/AstralisAST.png', 33);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (16, 101383793572656373, 'excel', 'EXCEL', 'XL', 'http://static.lolesports.com/teams/Excel_FullColor2.png', 'http://static.lolesports.com/teams/Excel_FullColor1.png', 'http://static.lolesports.com/teams/ExcelXL.png', 33);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (17, 101383793574360315, 'rogue', 'Rogue', 'RGE', 'http://static.lolesports.com/teams/1631819715136_rge-2021-worlds.png', NULL, 'http://static.lolesports.com/teams/1632941190948_RGE.png', 33);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (18, 101388912911039804, 'thunder-talk-gaming', 'Thunder Talk Gaming', 'TT', 'http://static.lolesports.com/teams/TT-FullonDark.png', 'http://static.lolesports.com/teams/TT-FullonLight.png', 'http://static.lolesports.com/teams/TTTT.png', 35);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (19, 101388912914513220, 'victory-five', 'Victory Five', 'V5', 'http://static.lolesports.com/teams/1592592149333_VictoryFiveV5-01-FullonDark.png', 'http://static.lolesports.com/teams/1592592149336_VictoryFiveV5-03-FullonLight.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/victory-five-ha9mq1rv.png', 35);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (20, 101422616509070746, 'galatasaray-espor', 'Galatasaray Espor', 'GS', 'http://static.lolesports.com/teams/1631820533570_galatasaray-2021-worlds.png', 'http://static.lolesports.com/teams/1631820533572_galatasaray-2021-worlds.png', 'http://static.lolesports.com/teams/1632941006301_GalatasarayGS.png', 39);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (21, 101428372598668846, 'burning-core', 'Burning Core', 'BC', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/burning-core-7q0431w1.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/burning-core-8a63k0iu.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/burning-core-fnmfa2td.png', 40);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (22, 101428372600307248, 'rascal-jester', 'Rascal Jester', 'RJ', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/rascal-jester-e0g6cud0.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/rascal-jester-g32ay08v.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/rascal-jester-guqjh8kb.png', 40);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (23, 101428372602011186, 'v3-esports', 'V3 Esports', 'V3', 'http://static.lolesports.com/teams/v3_500x500.png', 'http://static.lolesports.com/teams/v3_500x500.png', NULL, 40);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (24, 101428372603715124, 'crest-gaming-act', 'Crest Gaming Act', 'CGA', 'http://static.lolesports.com/teams/1630058341510_cga_512px.png', 'http://static.lolesports.com/teams/1630058341513_cga_512px.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/crest-gaming-act-7pkgpqa.png', 40);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (25, 101428372605353526, 'sengoku-gaming', 'Sengoku Gaming', 'SG', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/sengoku-gaming-ikyxjlfn.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/sengoku-gaming-gnat0l9c.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/sengoku-gaming-3rd8ifie.png', 40);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (26, 101428372607057464, 'axiz', 'AXIZ', 'AXZ', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/axiz-frilmkic.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/axiz-fpemv4d2.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/axiz-9hiwgh3l.png', 40);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (27, 101428372830010965, 'alpha-esports', 'Alpha Esports', 'ALF', 'http://static.lolesports.com/teams/1592588479686_AlphaEsportsALF-01-FullonDark.png', 'http://static.lolesports.com/teams/1592588479688_AlphaEsportsALF-03-FullonLight.png', NULL, 4);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (28, 101978171843206569, 'vega-squadron', 'Vega Squadron', 'VEG', 'http://static.lolesports.com/teams/vega.png', 'http://static.lolesports.com/teams/vega.png', NULL, 41);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (29, 102141671181705193, 'michigan-state-university', 'Michigan State University', 'MSU', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/michigan-state-university-au4vndaf.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/michigan-state-university-c5mv9du0.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (30, 102141671182557163, 'university-of-illinois', 'University of Illinois', 'UI', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/university-of-illinois-bwvscsri.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/university-of-illinois-b3jros5r.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (31, 102141671183409133, 'maryville-university', 'Maryville University', 'MU', 'http://static.lolesports.com/teams/1647541915472_200x200_MU_Logo.png', 'http://static.lolesports.com/teams/1647541915475_200x200_MU_Logo.png', NULL, 28);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (32, 102141671185047537, 'uci-esports', 'UCI Esports', 'UCI', 'http://static.lolesports.com/teams/1641604280633_UCI.png', 'http://static.lolesports.com/teams/1641548061305_LOLESPORTSICON.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (33, 102141671185899507, 'university-of-western-ontario', 'University of Western Ontario', 'UWO', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/university-of-western-ontario-9q0nn3lw.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/university-of-western-ontario-6csb5dft.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (34, 102141671186685941, 'university-of-waterloo', 'University of Waterloo', 'UW', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/university-of-waterloo-2wuni11l.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/university-of-waterloo-aghmypqf.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (35, 102141671187668983, 'nc-state-university', 'NC State University', 'NCSU', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/nc-state-university-it42b898.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/nc-state-university-6ey19n1w.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (36, 102235771678061291, 'fastpay-wildcats', 'fastPay Wildcats', 'IW', 'http://static.lolesports.com/teams/fastpay-wildcats.png', 'http://static.lolesports.com/teams/fastpay-wildcats.png', NULL, 39);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (37, 102747101565183056, 'nongshim-redforce', 'NongShim REDFORCE', 'NS', 'http://static.lolesports.com/teams/NSFullonDark.png', 'http://static.lolesports.com/teams/NSFullonLight.png', 'http://static.lolesports.com/teams/NongshimRedForceNS.png', 34);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (38, 102787200120306562, 'mousesports', 'Mousesports', 'MOUZ', 'http://static.lolesports.com/teams/1639486346996_PRM_MOUZ-FullColorDarkBG.png', 'http://static.lolesports.com/teams/1639486346999_PRM_MOUZ-FullColorDarkBG.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (39, 102787200124959636, 'crvena-zvezda-esports', 'Crvena Zvezda Esports', 'CZV', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/crvena-zvezda-esports-ddtlzzhd.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/crvena-zvezda-esports-ddtlzzhd.png', NULL, 1);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (40, 102787200126663579, 'giants', 'Giants', 'GIA', 'http://static.lolesports.com/teams/1641412992057_escudowhite.png', 'http://static.lolesports.com/teams/1641412992058_escudo_black.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (41, 102787200129022886, 'esuba', 'eSuba', 'ESB', 'http://static.lolesports.com/teams/1629209489523_esuba_full_pos.png', 'http://static.lolesports.com/teams/1629209489525_esuba_full_pos.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (42, 102787200130988976, 'asus-rog-elite', 'ASUS ROG Elite', 'ASUS', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/asus-rog-elite-iouou6l.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/asus-rog-elite-cz4z103n.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (43, 102787200132955066, 'for-the-win-esports', 'For The Win Esports', 'FTW', 'http://static.lolesports.com/teams/LPLOL_FTW-Logo1.png', 'http://static.lolesports.com/teams/LPLOL_FTW-Logo1.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (44, 102787200134790084, 'hma-fnatic-rising', 'HMA Fnatic Rising', 'FNCR', 'http://static.lolesports.com/teams/NLC_FNCR-logo.png', 'http://static.lolesports.com/teams/NLC_FNCR-logo.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (45, 102787200136756173, 'berlin-international-gaming', 'Berlin International Gaming', 'BIG', 'http://static.lolesports.com/teams/BIG-Logo-2020-White1.png', 'http://static.lolesports.com/teams/BIG-Logo-2020-White1.png', NULL, 7);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (46, 102787200138722262, 'devilsone', 'Devils.One', 'DV1', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/devilsone-bfe3xkh.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/devilsone-dmj5ivct.png', NULL, 6);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (47, 102787200143309800, 'ensure', 'eNsure', 'EN', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/ensure-5hi6e2cg.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/ensure-fehdkert.png', NULL, 1);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (48, 102787200145472495, 'defusekids', 'Defusekids', 'DKI', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/defusekids-finmimok.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/defusekids-wu2z0pj.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (49, 102787200147504121, 'campus-party-sparks', 'Campus Party Sparks', 'SPK', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/campus-party-sparks-5h2d1rjh.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/campus-party-sparks-72ccff49.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (50, 102787200149928963, 'we-love-gaming', 'We Love Gaming', 'WLG', 'http://static.lolesports.com/teams/WLGlogo.png', 'http://static.lolesports.com/teams/WLGlogo.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (51, 102787200151698443, 'vitalitybee', 'Vitality.Bee', 'VITB', 'http://static.lolesports.com/teams/Vitality-logo-color-outline-rgb.png', 'http://static.lolesports.com/teams/Vitality-logo-color-outline-rgb.png', NULL, 1);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (52, 102787200153467923, 'bcn-squad', 'BCN Squad', 'BCN', 'http://static.lolesports.com/teams/SL_BCN-Logo_White.png', 'http://static.lolesports.com/teams/SL_BCN-Logo_Dark.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (53, 102787200155434012, 'jdxl', 'JD|XL', 'JDXL', 'http://static.lolesports.com/teams/1641489535868_jdxl.png', NULL, NULL, 9);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (54, 102787200157400101, 'falkn', 'FALKN', 'FKN', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/falkn-j72aqsqk.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/falkn-dhvtpixb.png', NULL, 1);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (55, 102787200159169580, 'godsent', 'Godsent', 'GOD', 'http://static.lolesports.com/teams/NLC_GOD-light.png', 'http://static.lolesports.com/teams/NLC_GOD-dark.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (56, 102825747701670848, 'azules-esports', 'Azules Esports', 'UCH', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/azules-esports-ak2khbqa.png', NULL, 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/azules-esports-e8yjxxki.png', NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (57, 103461966951059521, 'evil-geniuses', 'Evil Geniuses', 'EG', 'http://static.lolesports.com/teams/1592590374862_EvilGeniusesEG-01-FullonDark.png', 'http://static.lolesports.com/teams/1592590374875_EvilGeniusesEG-03-FullonLight.png', 'http://static.lolesports.com/teams/1590003096057_EvilGeniusesEG.png', 32);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (58, 103461966965149786, 'mad-lions', 'MAD Lions', 'MAD', 'http://static.lolesports.com/teams/1631819614211_mad-2021-worlds.png', 'http://static.lolesports.com/teams/1592591395341_MadLionsMAD-03-FullonLight.png', 'http://static.lolesports.com/teams/MAD.png', 33);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (59, 103461966971048042, 'eg-academy', 'EG Academy', 'EG', 'http://static.lolesports.com/teams/1592590391188_EvilGeniusesEG-01-FullonDark.png', 'http://static.lolesports.com/teams/1592590391200_EvilGeniusesEG-03-FullonLight.png', 'http://static.lolesports.com/teams/1590003135776_EvilGeniusesEG.png', 28);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (60, 103461966975897718, 'imt-academy', 'IMT Academy', 'IMT', 'http://static.lolesports.com/teams/imt-new-color.png', 'http://static.lolesports.com/teams/imt-new-color.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/immortals-academy-hmxmnvhe.png', 28);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (61, 103461966981927044, 'dig-academy', 'DIG Academy', 'DIG', 'http://static.lolesports.com/teams/DIG-FullonDark.png', 'http://static.lolesports.com/teams/DIG-FullonLight.png', 'http://static.lolesports.com/teams/DignitasDIG.png', 28);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (62, 103461966986776720, 'ultra-prime', 'Ultra Prime', 'UP', 'http://static.lolesports.com/teams/ultraprime.png', 'http://static.lolesports.com/teams/ultraprime.png', NULL, 35);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (63, 103495716836203404, '5-ronin', '5 Ronin', '5R', 'http://static.lolesports.com/teams/5R_LOGO.png', 'http://static.lolesports.com/teams/5R_LOGO.png', NULL, 39);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (100, 104211666442891296, 'ogaming', 'O''Gaming', 'OGA', 'http://static.lolesports.com/teams/1590143833802_Ays7Gjmu_400x400.jpg', NULL, NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (64, 103495716886587312, 'besiktas', 'Beşiktaş', 'BJK', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/besiktas-e-sports-club-dlw48ntu.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/besiktas-e-sports-club-6ttscu28.png', NULL, 39);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (65, 103535282113853330, '5-ronin-akademi', '5 Ronin Akademi', '5R', 'http://static.lolesports.com/teams/5R_LOGO.png', 'http://static.lolesports.com/teams/5R_LOGO.png', NULL, 2);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (66, 103535282119620510, 'fukuoka-softbank-hawks-gaming', 'Fukuoka SoftBank HAWKS gaming', 'SHG', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/fukuoka-softbank-hawks-gaming-b99n2uq2.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/fukuoka-softbank-hawks-gaming-4i3ympnq.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/fukuoka-softbank-hawks-gaming-4fl2jmuh.png', 40);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (67, 103535282124208038, 'pentanetgg', 'Pentanet.GG', 'PGG', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/pentanetgg-3vnqnv03.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/pentanetgg-3d4g4sbh.png', NULL, 16);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (68, 103535282135552642, 'papara-supermassive-blaze-akademi', 'Papara SuperMassive Blaze Akademi', 'SMB', 'http://static.lolesports.com/teams/1628521896643_SMBA_WHITE.png', 'http://static.lolesports.com/teams/1628521896646_SMBA_BLACK.png', NULL, 2);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (69, 103535282138043022, 'fenerbahce-espor-akademi', 'Fenerbahçe Espor Akademi', 'FB', 'http://static.lolesports.com/teams/1642680283028_BANPICK_FB.png', 'http://static.lolesports.com/teams/1642680283035_BANPICK_FB.png', NULL, 2);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (70, 103535282140533402, 'besiktas-akademi', 'Beşiktaş Akademi', 'BJK', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/besiktas-akademi-6dlbk21d.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/besiktas-akademi-fobrhai9.png', NULL, 2);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (71, 103535282143744679, 'dark-passage-akademi', 'Dark Passage Akademi', 'DP', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/dark-passage-akademi-9ehs6q0l.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/dark-passage-akademi-h4x5hq6.png', NULL, 2);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (72, 103535282146169523, 'info-yatrm-aurora-akademi', 'Info Yatırım Aurora Akademi', 'AUR', 'http://static.lolesports.com/teams/1642680351930_BANPICK_AUR.png', 'http://static.lolesports.com/teams/1642680351936_BANPICK_AUR.png', NULL, 2);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (73, 103535282148790975, 'galakticos-akademi', 'GALAKTICOS Akademi', 'GAL', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/galakticos-akademi-4x1ww2pc.png', 'https://lolstatic-a.akamaihd.net/esports-assets/production/team/galakticos-akademi-dv3kn0pg.png', NULL, 2);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (74, 103535282158162659, 'fastpay-wildcats-akademi', 'fastPay Wildcats Akademi', 'IW', 'http://static.lolesports.com/teams/1582880891336_IW.png', 'http://static.lolesports.com/teams/1582880891351_IW.png', NULL, 2);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (75, 103877554248683116, 'schalke-04-evolution', 'Schalke 04 Evolution', 'S04E', 'http://static.lolesports.com/teams/S04_Standard_Logo1.png', 'http://static.lolesports.com/teams/S04_Standard_Logo1.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (76, 103877589042434434, 'gamerlegion', 'GamerLegion', 'GL', 'http://static.lolesports.com/teams/1585046217463_220px-Team_GamerLegionlogo_square.png', NULL, NULL, 1);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (77, 103877625775457850, 'movistar-riders', 'Movistar Riders', 'MRS', 'http://static.lolesports.com/teams/1585046777741_220px-Movistar_Riderslogo_square.png', NULL, NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (78, 103877675241047720, 'ldlc-ol', 'LDLC OL', 'LDLC', 'http://static.lolesports.com/teams/LFL-LDLC-logo.png', 'http://static.lolesports.com/teams/LFL-LDLC-logo.png', NULL, 1);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (79, 103877737868887783, 'saim-se', 'SAIM SE', 'SSB', 'http://static.lolesports.com/teams/1585048488568_220px-SAIM_SElogo_square.png', 'http://static.lolesports.com/teams/1585048488582_220px-SAIM_SElogo_square.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (80, 103877756742242918, 'racoon', 'Racoon', 'RCN', 'http://static.lolesports.com/teams/1585048776551_220px-Racoon_(Italian_Team)logo_square.png', 'http://static.lolesports.com/teams/1585048776564_220px-Racoon_(Italian_Team)logo_square.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (81, 103877774634323825, 'ydn-gamers', 'YDN Gamers', 'YDN', 'http://static.lolesports.com/teams/1587638409857_LOGO_YDN_-trasp.png', 'http://static.lolesports.com/teams/1587638409876_LOGO_YDN_-trasp.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (82, 103877879209300619, 'vipers-inc', 'Vipers Inc', 'VIP', 'http://static.lolesports.com/teams/1585050644953_220px-Vipers_Inclogo_square.png', 'http://static.lolesports.com/teams/1585050644968_220px-Vipers_Inclogo_square.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (83, 103877891572305836, 'team-singularity', 'Team Singularity', 'SNG', 'http://static.lolesports.com/teams/NLC_SNG-light.png', 'http://static.lolesports.com/teams/NLC_SNG-logo.png', NULL, 9);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (84, 103877908090914662, 'kenty', 'Kenty', 'KEN', 'http://static.lolesports.com/teams/1585051086000_220px-Kentylogo_square.png', 'http://static.lolesports.com/teams/1585051086014_220px-Kentylogo_square.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (85, 103877925817094140, 'pigsports', 'PIGSPORTS', 'PIG', 'http://static.lolesports.com/teams/PIGSPORTS_PIG-Logo1.png', 'http://static.lolesports.com/teams/PIGSPORTS_PIG-Logo1.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (86, 103877951616192529, 'cyber-gaming', 'Cyber Gaming', 'CG', 'http://static.lolesports.com/teams/1585051749524_220px-Cyber_Gaminglogo_square.png', 'http://static.lolesports.com/teams/1585051749529_220px-Cyber_Gaminglogo_square.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (87, 103877976717529187, 'intrepid-fox-gaming', 'Intrepid Fox Gaming', 'IF', 'http://static.lolesports.com/teams/1585052132267_220px-Intrepid_Fox_Gaminglogo_square.png', 'http://static.lolesports.com/teams/1585052132281_220px-Intrepid_Fox_Gaminglogo_square.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (88, 103878020539746273, 'egn-esports', 'EGN Esports', 'EGN', 'http://static.lolesports.com/teams/LPLOL_EGN-Logo1.png', 'http://static.lolesports.com/teams/LPLOL_EGN-Logo1.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (89, 103935421249833954, 'mad-lions-madrid', 'MAD Lions Madrid', 'MADM', 'http://static.lolesports.com/teams/SL_MADM-Logo_white.png', 'http://static.lolesports.com/teams/SL_MADM-Logo_dark.png', NULL, 5);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (90, 103935446548920777, 'misfits-premier', 'Misfits Premier', 'MSFP', 'http://static.lolesports.com/teams/LFL-MSFP-logo.png', 'http://static.lolesports.com/teams/LFL-MSFP-logo.png', NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (91, 103935468920814040, 'gamersorigin', 'GamersOrigin', 'GO', 'http://static.lolesports.com/teams/1588178480033_logoGO_2020_G_Blanc.png', 'http://static.lolesports.com/teams/1588178480035_logoGO_2020_G_Noir.png', NULL, 11);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (92, 103935523328473675, 'k1ck-neosurf', 'K1CK Neosurf', 'K1', 'http://static.lolesports.com/teams/1585930223604_K1ck_Neosurflogo_square.png', NULL, NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (93, 103935530333072898, 'ago-rogue', 'AGO Rogue', 'RGO', 'http://static.lolesports.com/teams/1585930330127_AGO_ROGUElogo_square.png', NULL, NULL, 1);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (94, 103935567188806885, 'energypot-wizards', 'Energypot Wizards', 'EWIZ', 'http://static.lolesports.com/teams/1585930892362_Energypot_Wizardslogo_square.png', NULL, NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (95, 103935642731826448, 'sector-one', 'Sector One', 'S1', 'http://static.lolesports.com/teams/1641288621852_1024x1024_sector_one_nameless_white.png', 'http://static.lolesports.com/teams/1641288621854_1024x1024_sector_one_nameless_black.png', NULL, 19);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (96, 103963647433204351, 'm19', 'M19', 'M19', 'http://static.lolesports.com/teams/1586359360406_M19logo_square.png', NULL, NULL, NULL);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (97, 103963715924353674, 'dragon-army', 'Dragon Army', 'DA', 'http://static.lolesports.com/teams/1586360405423_440px-Dragon_Armylogo_square.png', NULL, NULL, 41);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (98, 103963753080578719, 'crowcrowd-moscow', 'CrowCrowd Moscow', 'CC', 'http://static.lolesports.com/teams/Logo_CC.png', NULL, NULL, 41);
INSERT INTO dbo.team (id, ext_id, slug, name, code, image_url, alt_image_url, bg_image_url, home_league) VALUES (99, 104202382255290736, 'rensga', 'RENSGA', 'RNS', 'http://static.lolesports.com/teams/LogoRensgaEsports.png', 'http://static.lolesports.com/teams/LogoRensgaEsports.png', 'http://static.lolesports.com/teams/RensgaRNS.png', 37);
SET IDENTITY_INSERT dbo.team OFF;

-- Values for tournament table
SET IDENTITY_INSERT dbo.tournament ON;
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (1, 107893386210553711, 'european_masters_spring_2022_main_event', '2022-04-13', '2022-05-08', 1);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (2, 107530554766055254, 'lla_opening_2022', '2022-01-28', '2022-04-17', 3);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (3, 107693721179065689, 'pcs_2022_spring', '2022-02-11', '2022-04-18', 4);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (4, 107468241207873310, 'superliga_2022_spring', '2022-01-09', '2022-05-01', 5);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (5, 107416436272657995, 'ultraliga_2022_spring', '2022-01-01', '2022-05-01', 6);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (6, 107417741193036913, 'prime_2022_spring', '2022-01-01', '2022-05-01', 7);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (7, 107457033672415830, 'pg_spring', '2022-01-17', '2022-05-01', 8);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (8, 107417432877679361, 'nlc_2022_spring', '2022-01-01', '2022-05-15', 9);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (9, 107468370558963709, 'lfl_2022_spring', '2022-01-09', '2022-05-01', 11);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (10, 107565607659994755, 'cblol_academy_2022', '2022-01-24', '2022-04-18', 15);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (11, 107439320897210747, 'lco_spring_2022', '2022-01-23', '2022-04-29', 16);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (12, 107563481236862420, 'eslol_spring', '2022-01-16', '2022-05-01', 19);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (13, 107682708465517027, 'discover_volcano_league_opening_2022', '2022-01-25', '2022-04-16', 22);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (14, 107728324355999617, 'master_flow_league_opening_2022', '2022-01-26', '2022-04-24', 24);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (15, 107677841285321565, 'honor_league_opening_2022', '2022-01-24', '2022-04-16', 25);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (16, 107921288851375933, 'proving_grounds_spring_2022', '2022-03-16', '2022-04-16', 28);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (17, 108097587668586485, 'tft_emea_lcq_2022', '2022-04-16', '2022-04-16', 29);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (18, 107458367237283414, 'lcs_spring_2022', '2022-02-04', '2022-04-25', 32);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (19, 107417059262120466, 'lec_2022_spring', '2022-01-01', '2022-05-15', 33);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (20, 107417779630700437, 'lpl_spring_2022', '2022-01-10', '2022-05-01', 35);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (21, 107405837336179496, 'cblol_2022_split1', '2022-01-22', '2022-04-23', 37);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (22, 107417471555810057, 'lcl_spring_2022', '2022-02-11', '2022-04-16', 41);
INSERT INTO dbo.tournament (id, ext_id, slug, start_date, end_date, league) VALUES (23, 107418086627198298, 'lcs_academy_2022_spring', '2022-01-19', '2022-05-31', 42);
SET IDENTITY_INSERT dbo.tournament OFF;
";