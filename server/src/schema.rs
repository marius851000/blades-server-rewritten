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
    users (id) {
        id -> Uuid,
        secret_id -> Uuid,
        data -> Jsonb,
    }
}

diesel::joinable!(characters -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(characters, users,);
