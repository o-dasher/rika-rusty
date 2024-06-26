CREATE TABLE discord_guild (
    id BIGINT PRIMARY KEY
);

CREATE TABLE discord_channel (
    id BIGINT PRIMARY KEY
);

CREATE TABLE discord_user (
    id BIGINT PRIMARY KEY,
    osu_user_id BIGINT 
);

CREATE TABLE osu_user (
    id BIGINT PRIMARY KEY
);

CREATE TABLE osu_score_identifier (
    score_id BIGINT NOT NULL,
    mode SMALLINT NOT NULL,

    PRIMARY KEY (score_id, mode)
);

CREATE TABLE osu_score (
    osu_user_id BIGINT NOT NULL,

    mods INT NOT NULL,
    map_id INT NOT NULL,

    PRIMARY KEY (score_id, mode),
    FOREIGN KEY (osu_user_id) REFERENCES osu_user (id) ON DELETE CASCADE
) INHERITS (osu_score_identifier);

CREATE TABLE osu_performance_base (
    FOREIGN KEY (score_id, mode) REFERENCES osu_score (score_id, mode) ON DELETE CASCADE
) INHERITS (osu_score_identifier);

CREATE TABLE osu_performance (
    aim FLOAT NOT NULL,
    speed FLOAT NOT NULL,
    accuracy FLOAT NOT NULL,
    flashlight FLOAT NOT NULL,
    overall FLOAT NOT NULL,

    CHECK (mode = 0)
) INHERITS (osu_performance_base);

CREATE TABLE taiko_performance (
    accuracy FLOAT NOT NULL,
    difficulty FLOAT NOT NULL,
    overall FLOAT NOT NULL,

    CHECK (mode = 1)
) INHERITS (osu_performance_base);

CREATE TABLE mania_performance (
    difficulty FLOAT NOT NULL,
    overall FLOAT NOT NULL,

    CHECK (mode = 3)
) INHERITS (osu_performance_base);

CREATE TABLE booru_setting (
    id SERIAL PRIMARY KEY,

    guild_id BIGINT UNIQUE,
    user_id BIGINT UNIQUE,
    channel_id BIGINT UNIQUE,

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
