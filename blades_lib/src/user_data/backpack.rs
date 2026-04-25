use std::collections::{HashMap, HashSet};

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
impl Items {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Backpack {
    pub stackable_items: StackableItems,
    pub items: Items,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct BackpackUpdate {
    pub stackable_items: StackableItems,
    pub items: Items,
    pub removed_items: HashSet<Uuid>,
    pub removed_stackable_items: HashSet<Uuid>,
}

impl Backpack {
    pub fn generate_client_update(&self, tracker: &BackpackChangeTracker) -> BackpackUpdate {
        let mut update = BackpackUpdate::default();

        for changed_stackable_id in &tracker.stackable_items {
            if let Some(item) = self.stackable_items.0.get(changed_stackable_id) {
                update
                    .stackable_items
                    .0
                    .insert(*changed_stackable_id, item.clone());
            } else {
                update.removed_stackable_items.insert(*changed_stackable_id);
            }
        }

        for changed_item_id in &tracker.items {
            if let Some(item) = self.items.0.get(changed_item_id) {
                update.items.0.insert(*changed_item_id, item.clone());
            } else {
                update.removed_items.insert(*changed_item_id);
            }
        }
        update
    }
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EquippedItems(
    /// the UUID is the slot, and NOT the item id
    pub HashMap<Uuid, SingleEquippedItem>,
);

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Loadout {
    pub equipped_items: EquippedItems,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct LoadoutUpdate {
    pub equipped_items: EquippedItems,
    pub unequipped_item_slots: HashSet<Uuid>,
}

impl Loadout {
    pub fn generate_client_update(&self, tracker: &LoadoutChangeTracker) -> LoadoutUpdate {
        let mut update = LoadoutUpdate::default();

        for updated_loadout in &tracker.modified_equipped_items {
            if let Some(item) = self.equipped_items.0.get(&updated_loadout) {
                update
                    .equipped_items
                    .0
                    .insert(*updated_loadout, item.clone());
            } else {
                update.unequipped_item_slots.insert(*updated_loadout);
            }
        }
        update
    }
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

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompleteInventoryUpdate {
    pub backpack: BackpackUpdate,
    pub loadout: LoadoutUpdate,
    pub treasury: Treasury,
    pub overflow_treasury: Treasury,
    pub backpack_version: u64,
    pub treasury_version: u64,
}

impl CompleteInventory {
    pub fn generate_client_update(
        &self,
        tracker: &InventoryChangeTracker,
    ) -> CompleteInventoryUpdate {
        CompleteInventoryUpdate {
            backpack_version: self.backpack_version,
            treasury_version: self.treasury_version,
            backpack: self
                .backpack
                .generate_client_update(&tracker.modified_backpack),
            loadout: self
                .loadout
                .generate_client_update(&tracker.modified_loadout),
            treasury: Treasury::default(),
            overflow_treasury: Treasury::default(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct BackpackChangeTracker {
    pub stackable_items: HashSet<Uuid>,
    pub items: HashSet<Uuid>,
}

#[derive(Default, Debug, Clone)]
pub struct LoadoutChangeTracker {
    pub modified_equipped_items: HashSet<Uuid>,
}

#[derive(Default, Debug, Clone)]
pub struct InventoryChangeTracker {
    pub modified_loadout: LoadoutChangeTracker,
    pub modified_backpack: BackpackChangeTracker,
    //TODO: treasury change tracker
}
