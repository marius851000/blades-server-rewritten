use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct Character {
    name: String,
}

pub struct UserSavedData {
    character: Character,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserAccount {
    pub gp_deviceids: HashSet<String>,
    /// The user id that is actually communicated with the client, and should be kept secret
    pub secret_id: Uuid,
}

impl UserAccount {
    pub fn create_new_user() -> Self {
        UserAccount {
            gp_deviceids: HashSet::default(),
            secret_id: Uuid::new_v4(),
        }
    }
}
