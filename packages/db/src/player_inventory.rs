use std::collections::HashMap;
/// This is a unified collection for all inventories. This includes
/// player inventories, npc inventories, shop inventories, etc. Trades between
/// inventories can be accomplished using a single lock on this table
///
use std::sync::Arc;

use anyhow::Result;
use redb::ReadableTable;
use redb::TableDefinition;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerInventory {
    pub player_id: String,
    pub items: HashMap<u8, (u64, u32)>,
}

/// we need to store the inventory contents, but also which item types
/// are where (for stacking), and where the next empty slot is (for new items)
const PLAYER_INVENTORY_TABLE: TableDefinition<(&str, u8), (u64, u32)> =
    TableDefinition::new("player_inventory");

impl PlayerInventory {
    pub fn init(db: &redb::Database) -> Result<()> {
        let write = db.begin_write()?;
        write.open_table(PLAYER_INVENTORY_TABLE)?;
        write.commit()?;
        Ok(())
    }

    pub fn new(player_id: String) -> Self {
        Self {
            player_id,
            items: HashMap::new(),
        }
    }

    pub fn load(db: &redb::Database, player_id: &str) -> Result<Self> {
        let mut out = Self {
            player_id: player_id.to_string(),
            items: HashMap::new(),
        };
        let read = db.begin_read()?;
        let inventory_table = read.open_table(PLAYER_INVENTORY_TABLE)?;
        for i in 0..=u8::MAX {
            if let Some(entry) = inventory_table.get((player_id, i))? {
                out.items.insert(i, entry.value());
            }
        }
        Ok(out)
    }

    pub fn insert(
        &mut self,
        db: Arc<redb::Database>,
        slot_index: u8,
        entry: (u64, u32),
    ) -> Result<()> {
        assert_ne!(entry.0, 0);
        let write = db.begin_write()?;
        {
            let mut inventory_table = write.open_table(PLAYER_INVENTORY_TABLE)?;
            if entry.1 == 0 {
                // all items dropped
                inventory_table.remove((self.player_id.as_str(), slot_index))?;
                self.items.remove(&slot_index);
            } else {
                inventory_table.insert((self.player_id.as_str(), slot_index), entry)?;
                self.items.insert(slot_index, entry);
            }
        }
        write.commit()?;
        Ok(())
    }

    pub fn player_picked_up(
        &mut self,
        db: Arc<redb::Database>,
        item_type: u64,
        count: u32,
    ) -> Result<Option<(u8, (u64, u32))>> {
        assert!(count > 0);
        let write = db.begin_write()?;
        let mut inventory_table = write.open_table(PLAYER_INVENTORY_TABLE)?;
        let mut empty_slot_maybe = None;
        let mut item_type_slot_maybe = None;
        for i in 0..=u8::MAX {
            if empty_slot_maybe.is_some() && item_type_slot_maybe.is_some() {
                break;
            }
            let inventory_slot = inventory_table.get((self.player_id.as_str(), i))?;
            if empty_slot_maybe.is_none() && inventory_slot.is_none() {
                empty_slot_maybe = Some(i);
            }
            if let Some(obj) = inventory_slot {
                let (slot_item_type, _count) = obj.value();
                if item_type_slot_maybe.is_none() {
                    if item_type == slot_item_type {
                        item_type_slot_maybe = Some(i);
                    }
                }
            }
        }
        // no space in inventory
        if empty_slot_maybe.is_none() && item_type_slot_maybe.is_none() {
            return Ok(None);
        }
        let slot_index = item_type_slot_maybe.unwrap_or(empty_slot_maybe.unwrap());
        let new_record = match inventory_table.get((self.player_id.as_str(), slot_index))? {
            Some(old) => {
                let mut new = old.value();
                assert_eq!(new.0, item_type);
                assert!(new.1 >= 1);
                new.1 += count;
                new
            }
            None => (item_type, count),
        };
        inventory_table.insert((self.player_id.as_str(), slot_index), new_record)?;

        drop(inventory_table);
        write.commit()?;

        Ok(Some((slot_index, new_record)))
    }
}
