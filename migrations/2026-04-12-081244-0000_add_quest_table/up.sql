CREATE TABLE quests (
    id UUID PRIMARY KEY,
    character_id UUID references characters(id) NOT NULL,
    info JSONB NOT NULL,
    generated_data JSONB NOT NULL
)
