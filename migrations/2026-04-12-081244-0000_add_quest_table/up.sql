CREATE TABLE quests (
    id          UUID              NOT NULL,
    character_id UUID            NOT NULL REFERENCES characters(id),
    info        JSONB             NOT NULL,
    generated_data JSONB          NOT NULL,
    PRIMARY KEY (id, character_id)
);
