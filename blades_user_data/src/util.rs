#[macro_export]
macro_rules! generate_map_to_vec_serialization {
    // ──────────────────────────────────────────────────────────────
    //  $prefix     - a short identifier (e.g. `backpack`)
    //  $inner      - a type that contains the inner value as field,
    //              will be flattened
    //  $id_field   - the field name that holds the Uuid (must be an
    //                identifier, e.g. `item_template_id`. It will be
    //                converted to camelCase).
    // ──────────────────────────────────────────────────────────────
    ($prefix:ident, $inner:ident, $id_field:ident) => {
        // 1️⃣  Entry type used only for (de)serialisation
        mod $prefix {
            use super::*;
            #[derive(Serialize, Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Entry {
                $id_field: Uuid,
                #[serde(flatten)]
                value: $inner,
            }

            // 2️⃣  Serializer that converts the map to a Vec<$prefix_entry>
            pub fn serialize<S: Serializer>(
                map: &HashMap<Uuid, $inner>,
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                $inner: Clone,
            {
                let entries: Vec<Entry> = map
                    .iter()
                    .map(|(id, value)| Entry {
                        value: value.clone(),
                        $id_field: *id,
                    })
                    .collect();
                entries.serialize(serializer)
            }

            // 3️⃣  Deserializer that rebuilds the map
            pub fn deserialize<'de, D: Deserializer<'de>>(
                deserializer: D,
            ) -> Result<HashMap<Uuid, $inner>, D::Error> {
                let entries: Vec<Entry> = Vec::deserialize(deserializer)?;
                let mut map = HashMap::new();
                for entry in entries {
                    map.insert(entry.$id_field, entry.value);
                }
                Ok(map)
            }
        }
    };
}
