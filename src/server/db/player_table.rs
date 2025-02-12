use std::fmt::Debug;

use anyhow::Result;
use redb::ReadTransaction;
use redb::ReadableTable;
use redb::TableDefinition;
use redb::WriteTransaction;
use serde::Deserialize;
use serde::Serialize;

pub const PLAYER_TABLE: TableDefinition<String, &[u8]> = TableDefinition::new("players");

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerRecord {
    pub id: String,
    pub username: String,
    pub current_map: String,
    pub experience: u64,
}

impl PlayerRecord {
    // Save a player to the database
    pub fn save(&self, tx: &mut WriteTransaction, player_id: String) -> Result<(), redb::Error> {
        let mut table = tx.open_table(PLAYER_TABLE)?;
        let bytes = bincode::serialize(self).unwrap();
        table.insert(player_id, bytes.as_slice())?;
        Ok(())
    }

    // Load a player from the database
    pub fn load(tx: &mut ReadTransaction, player_id: String) -> Result<Option<Self>, redb::Error> {
        let table = tx.open_table(PLAYER_TABLE)?;
        if let Some(bytes) = table.get(player_id)? {
            let player = bincode::deserialize(bytes.value()).unwrap();
            Ok(Some(player))
        } else {
            Ok(None)
        }
    }

    /// TODO: use a seperate table to avoid scanning
    pub fn player_by_name(
        tx: &mut ReadTransaction,
        username: &str,
    ) -> anyhow::Result<Option<Self>> {
        let table = tx.open_table(PLAYER_TABLE)?;
        for v in table.iter()? {
            let (_key, data) = v?;
            let player: Self = bincode::deserialize(data.value()).unwrap();
            if player.username == username {
                return Ok(Some(player));
            }
        }
        Ok(None)
    }
}
