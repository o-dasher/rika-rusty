CREATE TABLE osu_user (
    id BIGINT PRIMARY KEY
);

CREATE TABLE osu_score_identifier (
    score_id NUMERIC NOT NULL,
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
    overall FLOAT NOT NULL,
    FOREIGN KEY (score_id, mode) REFERENCES osu_score (score_id, mode) ON DELETE CASCADE
) INHERITS (osu_score_identifier);

CREATE TABLE osu_performance (
    aim FLOAT NOT NULL,
    speed FLOAT NOT NULL,
    accuracy FLOAT NOT NULL,
    flashlight FLOAT NOT NULL,

    CHECK (mode = 0)
) INHERITS (osu_performance_base);

CREATE TABLE taiko_performance (
    accuracy FLOAT NOT NULL,
    difficulty FLOAT NOT NULL,

    CHECK (mode = 1)
) INHERITS (osu_performance_base);

CREATE TABLE catch_performance (
    CHECK (mode = 2)
) INHERITS (osu_performance_base);

CREATE TABLE mania_performance (
    difficulty FLOAT NOT NULL,

    CHECK (mode = 3)
) INHERITS (osu_performance_base);


