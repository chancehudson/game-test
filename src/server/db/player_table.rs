use std::fmt::Debug;

use anyhow::Result;
use redb::ReadTransaction;
use redb::ReadableTable;
use redb::TableDefinition;
use redb::WriteTransaction;
use serde::Deserialize;
use serde::Serialize;

use crate::DB_HANDLER;

pub const PLAYER_TABLE: TableDefinition<String, Vec<u8>> = TableDefinition::new("players");

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
        table.insert(player_id, bytes)?;
        Ok(())
    }

    // Load a player from the database
    pub fn load(tx: &mut ReadTransaction, player_id: String) -> Result<Option<Self>, redb::Error> {
        let table = tx.open_table(PLAYER_TABLE)?;
        if let Some(bytes) = table.get(player_id)? {
            let player = bincode::deserialize(bytes.value().as_slice()).unwrap();
            Ok(Some(player))
        } else {
            Ok(None)
        }
    }

    pub async fn player_by_id(player_id: String) -> anyhow::Result<Option<Self>> {
        let read_tx = DB_HANDLER.read().await.db.begin_read()?;
        let table = read_tx.open_table(PLAYER_TABLE)?;
        if let Some(v) = table.get(player_id)? {
            let player_record: Self = bincode::deserialize(v.value().as_slice()).unwrap();
            Ok(Some(player_record))
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
            let player: Self = bincode::deserialize(data.value().as_slice()).unwrap();
            if player.username == username {
                return Ok(Some(player));
            }
        }
        Ok(None)
    }
}
