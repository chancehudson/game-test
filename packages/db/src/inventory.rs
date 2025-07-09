/// This is a unified collection for all inventories. This includes
/// player inventories, npc inventories, shop inventories, etc. Trades between
/// inventories can be accomplished using a single lock on this table
///
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum OwnerType {
    Player,
}

/*
 * We need not just to store the position of an item in an inventory, but also stacks of items.
 */
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct InventoryRecord {
    pub owner_id: String,
    pub owner_type: OwnerType,
    pub item_id: String,
    // supporting inventories of different sizes?
    pub slot_index: u8,
}

#[cfg(feature = "server")]
mod server_only {
    use super::*;

    use redb::TableDefinition;

    const INVENTORY_TABLE: TableDefinition<(&str, u8), InventoryRecord> =
        TableDefinition::new("inventory");

    impl InventoryRecord {}

    impl redb::Value for InventoryRecord {
        type SelfType<'a> = InventoryRecord;
        type AsBytes<'a> = Vec<u8>;

        fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
        where
            Self: 'b,
        {
            bincode::serialize(value).unwrap()
        }

        fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
        where
            Self: 'a,
        {
            bincode::deserialize(data).unwrap()
        }

        fn type_name() -> redb::TypeName {
            redb::TypeName::new("InventoryRecord")
        }

        fn fixed_width() -> Option<usize> {
            None
        }
    }
}
