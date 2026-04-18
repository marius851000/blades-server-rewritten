// @generated automatically by Diesel CLI.

diesel::table! {
    characters (id) {
        id -> Uuid,
        user_id -> Uuid,
        character -> Jsonb,
        data -> Jsonb,
        inventory -> Jsonb,
        wallet -> Jsonb,
    }
}

diesel::table! {
    quests (id, character_id) {
        id -> Uuid,
        character_id -> Uuid,
        info -> Jsonb,
        generated_data -> Jsonb,
        dungeon_state -> Nullable<Jsonb>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        secret_id -> Uuid,
        data -> Jsonb,
    }
}

diesel::joinable!(characters -> users (user_id));
diesel::joinable!(quests -> characters (character_id));

diesel::allow_tables_to_appear_in_same_query!(characters, quests, users,);
