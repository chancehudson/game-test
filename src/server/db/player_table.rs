use anyhow::Result;
use nanoid::nanoid;
use serde::Deserialize;
use serde::Serialize;
use sled::transaction::ConflictableTransactionError;
use sled::Transactional;

use crate::DB;

// player record
const PLAYER_TREE: &str = "players";
// map usernames to player id's for constant time access
const USERNAME_TREE: &str = "usernames";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerRecord {
    pub id: String,
    pub username: String,
    pub current_map: String,
    pub experience: u64,
}

impl PlayerRecord {
    pub async fn create(username: String) -> Result<Self> {
        let player_id = nanoid!();
        let player = Self {
            id: player_id.clone(),
            username,
            current_map: "eastwatch".to_string(),
            experience: 0,
        };
        let username_tree = DB.open_tree(USERNAME_TREE)?;
        let player_tree = DB.open_tree(PLAYER_TREE)?;
        (&username_tree, &player_tree)
            .transaction(|(username_tree, player_tree)| {
                if let None = username_tree.get(player.username.clone().into_bytes())? {
                    username_tree.insert(
                        player.username.clone().into_bytes(),
                        player.id.clone().into_bytes(),
                    )?;
                    player_tree.insert(
                        player.id.clone().into_bytes(),
                        bincode::serialize(&player).unwrap(),
                    )?;
                    Ok(())
                } else {
                    Err(ConflictableTransactionError::Abort(
                        "username already exists",
                    ))
                }
            })
            .map_err(|e| anyhow::anyhow!("Failed to create player: {}", e))?;
        Ok(player)
    }

    pub async fn change_map(player_id: String, from_map: &str, to_map: &str) -> Result<()> {
        let tree = DB.open_tree(PLAYER_TREE)?;
        tree.transaction(move |player_tree| {
            if let Some(player) = player_tree.get(&player_id)? {
                let mut player: PlayerRecord = bincode::deserialize(player.as_ref()).unwrap();
                if player.current_map != from_map {
                    return Err(ConflictableTransactionError::Abort(format!(
                        "player not in map: {from_map}"
                    )));
                }
                player.current_map = to_map.to_string();
                player_tree.insert(
                    player_id.clone().into_bytes(),
                    bincode::serialize(&player).unwrap(),
                )?;
                Ok(())
            } else {
                Err(ConflictableTransactionError::Abort(
                    "user not found".to_string(),
                ))
            }
        })
        .map_err(|e| anyhow::anyhow!("Failed to change player map: {}", e))?;
        Ok(())
    }

    // Load a player from the database
    pub async fn player_by_id(player_id: String) -> Result<Option<Self>> {
        let tree = DB.open_tree(PLAYER_TREE)?;
        if let Some(bytes) = tree.get(player_id)? {
            let player = bincode::deserialize(bytes.as_ref())?;
            Ok(Some(player))
        } else {
            Ok(None)
        }
    }

    /// TODO: use a seperate table to avoid scanning
    pub async fn player_by_name(username: &str) -> Result<Option<Self>> {
        let tree = DB.open_tree(PLAYER_TREE)?;
        for v in tree.iter() {
            let (_k, v) = v?;
            let player: Self = bincode::deserialize(v.as_ref())?;
            if player.username == username {
                return Ok(Some(player));
            }
        }
        Ok(None)
    }
}
