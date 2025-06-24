use anyhow::Result;
use nanoid::nanoid;
use serde::Deserialize;
use serde::Serialize;

use super::DEFAULT_MAP;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct PlayerRecord {
    pub id: String,
    pub username: String,
    pub current_map: String,
    pub current_health: u64,
}

#[cfg(feature = "server")]
mod server_only {

    use crate::db::AbilityExpRecord;
    use crate::db::PlayerStats;

    use super::*;
    use sled::IVec;
    use sled::Transactional;
    use sled::transaction::ConflictableTransactionError;

    // player record
    const PLAYER_TREE: &str = "players";
    // map usernames to player id's for constant time access
    const USERNAME_TREE: &str = "usernames";

    impl Into<IVec> for PlayerRecord {
        fn into(self) -> IVec {
            let bytes = bincode::serialize(&self).unwrap();
            IVec::from(bytes)
        }
    }

    impl From<&[u8]> for PlayerRecord {
        fn from(value: &[u8]) -> Self {
            bincode::deserialize(value.as_ref()).unwrap()
        }
    }

    impl From<IVec> for PlayerRecord {
        fn from(value: IVec) -> Self {
            bincode::deserialize(value.as_ref()).unwrap()
        }
    }

    impl PlayerRecord {
        pub fn set_health(db: sled::Db, player_id: &str, new_health: u64) -> Result<()> {
            let tree = db.open_tree(PLAYER_TREE)?;
            // if the record for player_id does not exist this will noop
            tree.fetch_and_update(player_id, |old_bytes| match old_bytes {
                Some(bytes) => {
                    let mut current_record = Self::from(bytes);
                    current_record.current_health = new_health;
                    Some(current_record)
                }
                None => None,
            })?;
            Ok(())
        }

        pub fn create(db: sled::Db, username: String) -> Result<Self> {
            let player_id = nanoid!();
            let player = Self {
                id: player_id.clone(),
                username,
                current_map: DEFAULT_MAP.to_string(),
                current_health: PlayerStats::default().max_health(),
            };
            let username_tree = db.open_tree(USERNAME_TREE)?;
            let player_tree = db.open_tree(PLAYER_TREE)?;
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

        pub fn change_map(
            db: sled::Db,
            player_id: &str,
            from_map: &str,
            to_map: &str,
        ) -> Result<PlayerRecord> {
            let tree = db.open_tree(PLAYER_TREE)?;
            tree.transaction(move |player_tree| {
                if let Some(player) = player_tree.get(&player_id)? {
                    let mut player: PlayerRecord = bincode::deserialize(player.as_ref()).unwrap();
                    println!("{:?}", player);
                    if player.current_map != from_map {
                        return Err(ConflictableTransactionError::Abort(format!(
                            "player not in map: {from_map}"
                        )));
                    }
                    player.current_map = to_map.to_string();
                    player_tree.insert(
                        player_id.to_string().into_bytes(),
                        bincode::serialize(&player).unwrap(),
                    )?;
                    Ok(player)
                } else {
                    Err(ConflictableTransactionError::Abort(
                        "user not found".to_string(),
                    ))
                }
            })
            .map_err(|e| anyhow::anyhow!("Failed to change player map: {}", e))
        }

        // Load a player from the database
        pub fn player_by_id(db: sled::Db, player_id: String) -> Result<Self> {
            let tree = db.open_tree(PLAYER_TREE)?;
            if let Some(bytes) = tree.get(&player_id)? {
                let player = bincode::deserialize(bytes.as_ref())?;
                Ok(player)
            } else {
                anyhow::bail!("DB: player not found for id {player_id}")
            }
        }

        /// TODO: use a seperate table to avoid scanning
        pub fn player_by_name(db: sled::Db, username: &str) -> Result<Option<Self>> {
            let tree = db.open_tree(PLAYER_TREE)?;
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
}
