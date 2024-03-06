DROP TABLE IF EXISTS users CASCADE;
DROP TABLE IF EXISTS discord_users CASCADE;
DROP TABLE IF EXISTS forgejo_users CASCADE;

CREATE TABLE IF NOT EXISTS users (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	username TEXT NOT NULL,
	password TEXT NOT NULL,
	permission_level INT NOT NULL DEFAULT 0,
	UNIQUE (id),
	UNIQUE (username)
);

CREATE TABLE IF NOT EXISTS discord_users (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	user_id BIGINT,
	discord_id TEXT NOT NULL,
	link_code TEXT,
	username TEXT NOT NULL,
	discriminator TEXT NOT NULL,
	global_name TEXT,
	UNIQUE (id),
	UNIQUE (discord_id),
	FOREIGN KEY (user_id)
		REFERENCES users (id)
);

CREATE TABLE IF NOT EXISTS forgejo_users (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	user_id BIGINT,
	forgejo_id BIGINT NOT NULL,
	link_code TEXT,
	username TEXT NOT NULL,
	global_name TEXT,
	active BOOLEAN NOT NULL,
	is_admin BOOLEAN NOT NULL,
	prohibit_login BOOLEAN NOT NULL,
	restricted BOOLEAN NOT NULL,
	UNIQUE (id),
	UNIQUE (forgejo_id),
	FOREIGN KEY (user_id)
		REFERENCES users (id)
);
