use std::collections::HashMap;
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

    pub fn drop(
        &mut self,
        db: Arc<redb::Database>,
        slot_index: u8,
        count: u32,
    ) -> Result<Option<(u64, u32)>> {
        #[cfg(debug_assertions)]
        assert_ne!(count, 0);
        let write = db.begin_write()?;
        let drop = {
            let mut inventory_table = write.open_table(PLAYER_INVENTORY_TABLE)?;
            let key = (self.player_id.as_str(), slot_index);
            let item_maybe = inventory_table.get(key)?.map(|v| v.value());
            match item_maybe {
                Some(item) => {
                    let drop_count = item.1.min(count);
                    #[cfg(debug_assertions)]
                    assert_ne!(drop_count, 0);
                    let mut item = item;
                    item.1 -= drop_count;
                    if item.1 == 0 {
                        inventory_table.remove(key)?;
                        self.items.remove(&slot_index);
                    } else {
                        inventory_table.insert(key, item)?;
                        self.items.insert(slot_index, item);
                    }
                    Some((item.0, drop_count))
                }
                None => {
                    #[cfg(debug_assertions)]
                    assert!(false);
                    None
                }
            }
        };
        write.commit()?;
        Ok(drop)
    }

    pub fn swap(&mut self, db: Arc<redb::Database>, indices: (u8, u8)) -> Result<()> {
        #[cfg(debug_assertions)]
        assert_ne!(indices.0, indices.1);
        let write = db.begin_write()?;
        {
            let mut inventory_table = write.open_table(PLAYER_INVENTORY_TABLE)?;
            let item_0 = inventory_table
                .remove((self.player_id.as_str(), indices.0))?
                .map(|v| v.value());
            let item_1 = inventory_table
                .remove((self.player_id.as_str(), indices.1))?
                .map(|v| v.value());
            match item_0 {
                Some(entry) => {
                    inventory_table.insert((self.player_id.as_str(), indices.1), entry)?;
                    self.items.insert(indices.1, entry);
                }
                None => {
                    self.items.remove(&indices.1);
                }
            }
            match item_1 {
                Some(entry) => {
                    inventory_table.insert((self.player_id.as_str(), indices.0), entry)?;
                }
                None => {
                    self.items.remove(&indices.0);
                }
            }
        }
        write.commit()?;
        Ok(())
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
                if item_type_slot_maybe.is_none()
                    && item_type == slot_item_type {
                        item_type_slot_maybe = Some(i);
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
