use serde::Deserialize;
use serde::Serialize;

use db::Ability;
use db::AbilityExpRecord;
use keind::prelude::*;

pub mod prelude;

mod data;
mod engine;
mod entity;
mod network;
mod system;

use prelude::*;

/// Inputs that may be applied to any entity.
#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct EntityInput {
    pub jump: bool,
    pub jump_down: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub crouch: bool,
    pub attack: bool,
    pub enter_portal: bool,
    pub show_emoji: bool,
    pub respawn: bool,
    pub pick_up: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    PlayerEnterPortal {
        player_id: String,
        entity_id: u128,
        // look at portals in the destination map and select the one farthest
        // to left or right automatically?
        from_map: String,
        to_map: String,
        requested_spawn_pos: Option<IVec2>,
    },
    // player entity id, ability
    PlayerAbilityExp(u128, Ability, u64),
    PlayerHealth(String, u64), // player health has changed through damage or healing
    Message(u128, String),     // message sent by an entity (npc or player)
    // player entity id
    PlayerPickUpRequest(u128),
    // player entity id, item type, count
    PlayerPickUp(String, u64, u32),
}

#[derive(EntitySystem, Debug, Clone, Serialize, Deserialize)]
pub enum EngineEntitySystem {
    Attach(AttachSystem),
    Disappear(DisappearSystem),
    PlayerExp(PlayerExpSystem),
    Gravity(GravitySystem),
    AtomicMove(AtomicMoveSystem),
    Weightless(WeightlessSystem),
    Invincible(InvincibleSystem),
}

#[derive(EngineEntity, Debug, Clone, Serialize, Deserialize)]
pub enum EngineEntity {
    Emoji(EmojiEntity),
    Item(ItemEntity),
    Message(MessageEntity),
    Mob(MobEntity),
    MobDamage(MobDamageEntity),
    MobSpawn(MobSpawnEntity),
    Npc(NpcEntity),
    Platform(PlatformEntity),
    Player(PlayerEntity),
    Portal(PortalEntity),
    Rect(RectEntity),
    Text(TextEntity),
}

/// A wrapper containing the game logic structures
/// exposed to the game by the engine
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeindGameLogic;

impl GameLogic for KeindGameLogic {
    type Entity = EngineEntity;
    type System = EngineEntitySystem;
    type Event = GameEvent;
    type Input = EntityInput;

    fn handle_game_events(
        engine: &mut GameEngine<Self>,
        game_events: &Vec<RefPointer<Self::Event>>,
    ) {
        for event in game_events {
            match &**event {
                GameEvent::PlayerEnterPortal {
                    player_id: _,
                    entity_id,
                    from_map: _,
                    to_map: _,
                    requested_spawn_pos: _,
                } => {
                    engine.remove_entity(*entity_id);
                }
                GameEvent::PlayerAbilityExp(_player_entity_id, _ability, _amount) => {}
                GameEvent::PlayerPickUpRequest(player_entity_id) => {
                    if let Some(player_entity) =
                        engine.entity_by_id::<PlayerEntity>(player_entity_id, None)
                    {
                        let mut item_id_maybe = None;
                        for item in engine.entities_by_type::<ItemEntity>() {
                            // if the player doesn't intersect the item ignore it
                            if item.rect().intersect(player_entity.rect()).is_empty() {
                                continue;
                            }
                            // otherwise pick up the item
                            // mark the item for removal
                            item_id_maybe = Some(item.id());

                            // register an event that will be handled by an external
                            // observer
                            engine.register_game_event(GameEvent::PlayerPickUp(
                                player_entity.player_id.clone(),
                                item.item_type,
                                item.count,
                            ));
                            break;
                        }
                        // remove the item immediately so if other pick up requests
                        // occured this step they don't pick up the same item
                        if let Some(item_id) = item_id_maybe {
                            engine.remove_entity_immediate(&item_id);
                        }
                    }
                }
                GameEvent::PlayerPickUp(_, _, _) => {}
                GameEvent::PlayerHealth(_, _) => {}
                GameEvent::Message(_, _) => {}
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct AnimationData {
    pub frame_count: usize,
    pub fps: usize,
    pub sprite_sheet: String,
    pub width: usize,
    pub height: usize,
}

impl AnimationData {
    pub fn static_data(sprite_sheet: &str, size: bevy_math::UVec2) -> Self {
        Self {
            frame_count: 1,
            fps: 1,
            sprite_sheet: sprite_sheet.to_string(),
            width: size.x as usize,
            height: size.y as usize,
        }
    }

    pub fn is_static(&self) -> bool {
        self.frame_count == 1
    }
}

// how many steps each client is behind the server
pub static STEP_DELAY: u64 = 60;
pub static STEPS_PER_SECOND: u32 = 60;
pub static STEP_LEN_S: f32 = 1.0 / STEPS_PER_SECOND as f32;

// Custom deserializer for Vec2
pub fn deserialize_vec2<'de, D>(deserializer: D) -> Result<bevy_math::IVec2, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let arr: [i32; 2] = Deserialize::deserialize(deserializer)?;
    Ok(bevy_math::IVec2::new(arr[0], arr[1]))
}
