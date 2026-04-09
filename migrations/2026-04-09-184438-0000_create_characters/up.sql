CREATE TABLE characters (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id) NOT NULL,
    character JSONB NOT NULL,
    data JSONB NOT NULL,

    /* limit to one character per user (until needing otherwise) */
    CONSTRAINT uq_characters_user_id
        UNIQUE (user_id)
)
