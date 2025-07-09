use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ItemRecord {
    pub id: String,
    pub item_id: u64,
    pub is_tradeable: bool,
    pub is_droppable: bool,
}

#[cfg(feature = "server")]
mod server_only {
    use super::*;

    use redb::ReadableTable;
    use redb::TableDefinition;

    const ITEM_TABLE: TableDefinition<&str, ItemRecord> = TableDefinition::new("items");

    impl redb::Value for ItemRecord {
        type SelfType<'a> = ItemRecord;
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
            redb::TypeName::new("ItemRecord")
        }

        fn fixed_width() -> Option<usize> {
            None
        }
    }
}
