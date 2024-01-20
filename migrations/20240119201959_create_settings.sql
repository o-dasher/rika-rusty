CREATE TABLE discord_guild (
	id INT PRIMARY KEY
);

CREATE TABLE discord_channel (
	id INT PRIMARY KEY,
	guild_id INT NOT NULL,

	FOREIGN KEY (guild_id) REFERENCES discord_guild(id)
);

CREATE TABLE discord_user (
	id INT PRIMARY KEY
);

CREATE TABLE booru_setting (
	id INT AUTO_INCREMENT PRIMARY KEY,

	guild_id INT,
	user_id INT,
	channel_id INT,

	FOREIGN KEY (guild_id) REFERENCES discord_guild(id) ON DELETE CASCADE,
	FOREIGN KEY (user_id) REFERENCES discord_user(id) ON DELETE CASCADE
);

CREATE TABLE booru_blacklisted_tag (
	id INT AUTO_INCREMENT PRIMARY KEY,

	booru_setting_id INT NOT NULL,
	blacklisted VARCHAR(255) NOT NULL,

	FOREIGN KEY (booru_setting_id) REFERENCES booru_setting(id) ON DELETE CASCADE
);
