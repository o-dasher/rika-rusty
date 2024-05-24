CREATE TABLE discord_guild (
    id NUMERIC PRIMARY KEY
);

CREATE TABLE discord_channel (
    id NUMERIC PRIMARY KEY,
    guild_id NUMERIC NOT NULL,

    FOREIGN KEY (guild_id) REFERENCES discord_guild (id)
);

CREATE TABLE discord_user (
    id NUMERIC PRIMARY KEY
);

CREATE TABLE booru_setting (
    id SERIAL PRIMARY KEY,

    guild_id NUMERIC UNIQUE,
    user_id NUMERIC UNIQUE,
    channel_id NUMERIC UNIQUE,

    FOREIGN KEY (guild_id) REFERENCES discord_guild (id) ON DELETE CASCADE,
    FOREIGN KEY (channel_id) REFERENCES discord_channel (id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES discord_user (id) ON DELETE CASCADE
);

CREATE TABLE booru_blacklisted_tag (
    booru_setting_id INT NOT NULL,
    blacklisted VARCHAR(255) NOT NULL,

    FOREIGN KEY (booru_setting_id) REFERENCES booru_setting (
        id
    ) ON DELETE CASCADE,
    PRIMARY KEY (blacklisted, booru_setting_id)
);
