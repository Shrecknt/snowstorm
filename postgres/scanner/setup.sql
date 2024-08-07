DROP TABLE IF EXISTS join_servers_players CASCADE;
DROP TABLE IF EXISTS servers CASCADE;
DROP TABLE IF EXISTS server_joins CASCADE;
DROP TABLE IF EXISTS players CASCADE;

CREATE TABLE IF NOT EXISTS servers (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	ip INT NOT NULL,
	port SMALLINT NOT NULL,
    version_name TEXT,
    version_protocol INT,
    max_players INT,
    online_players INT,
    online_anonymous_players INT,
    description TEXT,
    description_plain TEXT, -- description without formatting
    enforces_secure_chat BOOLEAN,
    previews_chat BOOLEAN,
    ping INT, -- two way ping
    geyser BOOLEAN,
	discovered BIGINT NOT NULL DEFAULT EXTRACT(epoch from now()),
	last_seen BIGINT NOT NULL DEFAULT EXTRACT(epoch from now()),
	UNIQUE (ip, port),
	UNIQUE (id)
);

CREATE TABLE IF NOT EXISTS server_joins (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    server_id BIGINT NOT NULL,
    online_mode BOOLEAN,
    whitelist BOOLEAN,
    bunger BOOLEAN,
    kick_message TEXT,
    flags BIT(8) NOT NULL DEFAULT B'00000000',
    error TEXT,
	first_joined BIGINT NOT NULL DEFAULT EXTRACT(epoch from now()),
	last_joined BIGINT NOT NULL DEFAULT EXTRACT(epoch from now()),
    CONSTRAINT fk_server
        FOREIGN KEY (server_id)
        REFERENCES servers(id)
);

CREATE TABLE IF NOT EXISTS players (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	uuid UUID NOT NULL,
	username TEXT NOT NULL,
    java_account BOOLEAN,
    bedrock_account BOOLEAN,
	UNIQUE (uuid, username)
);

CREATE TABLE IF NOT EXISTS join_servers_players (
	server_id BIGINT NOT NULL,
	player_id BIGINT NOT NULL,
    operator BOOLEAN,
    whitelisted BOOLEAN,
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
