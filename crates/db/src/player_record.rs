use anyhow::Result;
use nanoid::nanoid;
use redb::ReadableTable;
use redb::TableDefinition;
use serde::Deserialize;
use serde::Serialize;

use crate::DEFAULT_MAP;
use crate::PlayerStats;

const PLAYER_TABLE: TableDefinition<&str, PlayerRecord> = TableDefinition::new("players");
const USERNAME_TABLE: TableDefinition<&str, &str> = TableDefinition::new("player_usernames");

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct PlayerRecord {
    pub id: String,
    pub username: String,
    pub current_map: String,
    pub current_health: u64,
}

impl PlayerRecord {
    pub fn init(db: &redb::Database) -> Result<()> {
        let write = db.begin_write()?;
        write.open_table(PLAYER_TABLE)?;
        write.open_table(USERNAME_TABLE)?;
        write.commit()?;
        Ok(())
    }

    pub fn set_health(db: &redb::Database, player_id: &str, new_health: u64) -> Result<()> {
        let write = db.begin_write()?;
        let mut player_tree = write.open_table(PLAYER_TABLE)?;
        let player_record = match player_tree.get(player_id)? {
            Some(record) => {
                let mut player = record.value();
                player.current_health = new_health;
                player
            }
            None => {
                anyhow::bail!("Unable to find player with id: {player_id}");
            }
        };
        player_tree.insert(player_id, player_record)?;
        drop(player_tree);
        write.commit()?;
        Ok(())
    }

    pub fn create(db: &redb::Database, username: String) -> Result<Self> {
        let player_id = nanoid!();
        let write = db.begin_write()?;
        let mut username_table = write.open_table(USERNAME_TABLE)?;
        let mut player_table = write.open_table(PLAYER_TABLE)?;

        if let Some(_) = username_table.get(username.as_str())? {
            anyhow::bail!("username already in use!");
        }
        username_table.insert(username.as_str(), player_id.as_str())?;
        drop(username_table);

        let player = Self {
            id: player_id.clone(),
            username,
            current_map: DEFAULT_MAP.to_string(),
            current_health: PlayerStats::default().max_health(),
        };

        player_table.insert(player.id.as_str(), player.clone())?;
        drop(player_table);

        write.commit()?;

        Ok(player)
    }

    pub fn change_map(
        db: &redb::Database,
        player_id: &str,
        from_map: &str,
        to_map: &str,
    ) -> Result<PlayerRecord> {
        let write = db.begin_write()?;
        let mut player_table = write.open_table(PLAYER_TABLE)?;
        let player_record = match player_table.get(player_id)? {
            Some(player) => {
                let mut player = player.value();
                if player.current_map != from_map {
                    anyhow::bail!(
                        "Player attempting to change map: invalid from map expected {} got {from_map}",
                        player.current_map
                    );
                }
                player.current_map = to_map.to_string();
                player
            }
            None => {
                anyhow::bail!("Player not found for id: {player_id}");
            }
        };
        player_table.insert(player_id, player_record.clone())?;
        drop(player_table);
        write.commit()?;
        Ok(player_record)
    }

    // Load a player from the database
    pub fn player_by_id(db: &redb::Database, player_id: &str) -> Result<Self> {
        let read = db.begin_read()?;
        let player_table = read.open_table(PLAYER_TABLE)?;
        match player_table.get(player_id)? {
            Some(player) => Ok(player.value()),
            None => anyhow::bail!("Player not found for id {player_id}"),
        }
    }

    pub fn player_by_name(db: &redb::Database, username: &str) -> Result<Option<Self>> {
        let read = db.begin_read()?;
        let player_table = read.open_table(PLAYER_TABLE)?;
        let username_table = read.open_table(USERNAME_TABLE)?;
        match username_table.get(username)? {
            Some(player_id) => Ok(player_table
                .get(player_id.value())?
                .map(|player| player.value())),
            None => Ok(None),
        }
    }
}

impl redb::Value for PlayerRecord {
    type SelfType<'a> = PlayerRecord;
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
        redb::TypeName::new("PlayerRecord")
    }

    fn fixed_width() -> Option<usize> {
        None
    }
}
