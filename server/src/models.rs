use blades_user_data::{CompleteCharacter, CompleteData, UserAccount};
use diesel::prelude::*;
use uuid::Uuid;

use crate::json_db::JsonDbWrapper;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserDBEntry {
    pub id: Uuid,
    /// The user id that is actually communicated with the client, and should be kept secret
    pub secret_id: Uuid,
    pub data: JsonDbWrapper<UserAccount>,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::characters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CharacterDbEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub character: JsonDbWrapper<CompleteCharacter>,
    pub data: JsonDbWrapper<CompleteData>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::characters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CharacterDbEntryCharacterAndData {
    pub id: Uuid,
    pub user_id: Uuid,
    pub character: JsonDbWrapper<CompleteCharacter>,
    pub data: JsonDbWrapper<CompleteData>,
}
