use blades_lib::user_data::{
    CompleteCharacter, CompleteCharacterData, CompleteInventory, CompleteWallet,
    DungeonGeneratedData, DungeonState, Quest, UserAccount,
};
use diesel::prelude::*;
use uuid::Uuid;

use crate::{json_db::JsonDbWrapper, util::CharacterHolder};

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
    pub data: JsonDbWrapper<CompleteCharacterData>,
    pub wallet: JsonDbWrapper<CompleteWallet>,
    pub inventory: JsonDbWrapper<CompleteInventory>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::characters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CharacterDbEntryCharacterAlone {
    pub id: Uuid,
    pub user_id: Uuid,
    pub character: JsonDbWrapper<CompleteCharacter>,
}

impl CharacterHolder for CharacterDbEntryCharacterAlone {
    fn get_user_id(&self) -> &Uuid {
        &self.user_id
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::characters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CharacterDbEntryCharacterAndData {
    pub id: Uuid,
    pub user_id: Uuid,
    pub character: JsonDbWrapper<CompleteCharacter>,
    pub data: JsonDbWrapper<CompleteCharacterData>,
}

impl CharacterHolder for CharacterDbEntryCharacterAndData {
    fn get_user_id(&self) -> &Uuid {
        &self.user_id
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::characters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CharacterDbEntryWallet {
    pub user_id: Uuid,
    pub character: JsonDbWrapper<CompleteCharacter>,
    pub wallet: JsonDbWrapper<CompleteWallet>,
}

impl CharacterHolder for CharacterDbEntryWallet {
    fn get_user_id(&self) -> &Uuid {
        &self.user_id
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::characters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CharacterDbEntryInventory {
    pub user_id: Uuid,
    pub inventory: JsonDbWrapper<CompleteInventory>,
}

impl CharacterHolder for CharacterDbEntryInventory {
    fn get_user_id(&self) -> &Uuid {
        &self.user_id
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::characters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CharacterDbAlone {
    pub id: Uuid,
    pub user_id: Uuid,
}

impl CharacterHolder for CharacterDbAlone {
    fn get_user_id(&self) -> &Uuid {
        &self.user_id
    }
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::quests)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct QuestDbEntry {
    pub id: Uuid,
    pub character_id: Uuid,
    pub info: JsonDbWrapper<Quest>,
    pub generated_data: JsonDbWrapper<Option<DungeonGeneratedData>>,
    pub dungeon_state: Option<JsonDbWrapper<DungeonState>>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::quests)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct QuestDbEntryDungeonState {
    pub id: Uuid,
    pub dungeon_state: Option<JsonDbWrapper<DungeonState>>,
}
