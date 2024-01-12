DROP TABLE IF EXISTS users CASCADE;
DROP TABLE IF EXISTS discord_users CASCADE;

CREATE TABLE users (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	username TEXT NOT NULL,
	password TEXT NOT NULL,
	UNIQUE (id),
	UNIQUE (username)
);

CREATE TABLE discord_users (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	user_id BIGINT,
	discord_id TEXT NOT NULL,
	username TEXT NOT NULL,
	discriminator TEXT NOT NULL,
	global_name TEXT,
	link_code TEXT,
	UNIQUE (id),
	UNIQUE (discord_id),
	FOREIGN KEY (user_id)
		REFERENCES users (id)
);

DROP TABLE IF EXISTS join_table CASCADE;
DROP TABLE IF EXISTS servers CASCADE;
DROP TABLE IF EXISTS players CASCADE;

CREATE TABLE servers (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	ip INT NOT NULL,
	port SMALLINT NOT NULL,
    version_name TEXT,
    version_protocol INT,
    max_players INT,
    online_players INT,
    description TEXT,
    enforces_secure_chat BOOLEAN,
    previews_chat BOOLEAN,
	UNIQUE (ip, port),
	UNIQUE (id)
);

CREATE TABLE players (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	uuid UUID NOT NULL,
    username TEXT NOT NULL,
	UNIQUE (uuid)
);

CREATE TABLE join_table (
	server_id BIGINT NOT NULL,
	player_id BIGINT NOT NULL,
	discovered BIGINT NOT NULL DEFAULT EXTRACT(epoch from now()),
	last_seen BIGINT NOT NULL DEFAULT EXTRACT(epoch from now()),
	CONSTRAINT fk_player
		FOREIGN KEY (player_id) 
		REFERENCES players(id),
	CONSTRAINT fk_server
		FOREIGN KEY (server_id) 
		REFERENCES servers(id),
	CONSTRAINT join_pkey
		PRIMARY KEY (server_id, player_id)
);
