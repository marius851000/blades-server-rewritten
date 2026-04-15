use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

use crate::generate_map_to_vec_serialization;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct StackableItemStorageEntry {
    count: u64,
}

generate_map_to_vec_serialization!(
    stackable_item_serde,
    StackableItemStorageEntry,
    item_template_id
);

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StackableItems(
    #[serde(
        serialize_with = "stackable_item_serde::serialize",
        deserialize_with = "stackable_item_serde::deserialize"
    )]
    HashMap<Uuid, StackableItemStorageEntry>,
);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemSingleProperty {
    pub id: Uuid,
    pub tier: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "UPPERCASE")]
pub struct ItemPropertiesAll {
    pub enchanting: Vec<ItemSingleProperty>,
    pub grading: Vec<ItemSingleProperty>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub item_template_id: Uuid,
    pub tempering_level: u64,
    pub durability: f64,
    //TODO: do not serialize if there is no property
    pub properties: ItemPropertiesAll,
}

generate_map_to_vec_serialization!(items_serde, Item, id);

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Items(
    #[serde(
        serialize_with = "items_serde::serialize",
        deserialize_with = "items_serde::deserialize"
    )]
    pub HashMap<Uuid, Item>,
);

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Backpack {
    pub stackable_items: StackableItems,
    pub items: Items,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SingleEquippedItem {
    pub id: Uuid,
    /// This should be kept up to date with what slot it is in the parent EquippedItems
    pub slot: Uuid,
    #[serde(flatten)]
    pub item: Item,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EquippedItems(
    /// the UUID is the slot, and NOT the item id
    pub HashMap<Uuid, SingleEquippedItem>,
);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Loadout {
    pub equipped_items: EquippedItems,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Chest {
    // TODO: id is a number stored as a string. put all that in a hashmap with auto string conversion when implementing chests.
    id: String,
    tier: u64,
    level: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Treasury {
    chests: Vec<Chest>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompleteInventory {
    pub backpack: Backpack,
    pub loadout: Loadout,
    pub treasury: Treasury,
    // what is this overflow treasury responsible for?
    pub overflow_treasury: Treasury,
    pub backpack_version: u64,
    pub treasury_version: u64,
}
