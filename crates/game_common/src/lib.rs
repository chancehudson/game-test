use bevy_math::IVec2;
use keind::prelude::*;
use serde::Deserialize;
use serde::Serialize;

use db::Ability;
use db::AbilityExpRecord;

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

keind::engine_entity_system_enum!(
    KeindGameLogic,
    pub enum EngineEntitySystem {
        Attach(AttachSystem),
        Disappear(DisappearSystem),
        PlayerExp(PlayerExpSystem),
        Gravity(GravitySystem),
        AtomicMove(AtomicMoveSystem),
        Weightless(WeightlessSystem),
        Invincible(InvincibleSystem),
    }
);

keind::engine_entity_enum!(
    KeindGameLogic,
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
);

/// A wrapper containing the game logic structures
/// exposed to the game by the engine
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeindGameLogic {}
impl GameLogic for KeindGameLogic {
    type Entity = EngineEntity;
    type System = EngineEntitySystem;
    type Event = GameEvent;
    type Input = EntityInput;

    fn handle_game_events(engine: &GameEngine<Self>, game_events: &Vec<RefPointer<Self::Event>>) {
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
                GameEvent::PlayerAbilityExp(player_entity_id, ability, amount) => {
                    if let Some(player_entity) =
                        engine.entity_by_id::<PlayerEntity>(&player_entity_id, None)
                    {
                        engine.register_event(
                            None,
                            EngineEvent::SpawnSystem {
                                entity_id: *player_entity_id,
                                system_ptr: RefPointer::from(EngineEntitySystem::from(
                                    PlayerExpSystem {
                                        record: AbilityExpRecord {
                                            player_id: player_entity.player_id.clone(),
                                            amount: *amount,
                                            ability: ability.clone(),
                                        },
                                    },
                                )),
                                is_non_determinism: false,
                            },
                        );
                    } else {
                        println!("WARNING: received player exp event for non-existent entity");
                    }
                }
                GameEvent::PlayerPickUpRequest(_player_entity_id) => {
                    // if let Some(player_entity) = engine.entities.get(player_entity_id).cloned() {
                    //     let game_events_sender = engine.game_events.0.clone();
                    //     // there are quirks with using entities_by_type in the default handler
                    //     // see GameEngine::step
                    //     for item in engine
                    //         .entities_by_type::<ItemEntity>()
                    //         .cloned()
                    //         .collect::<Vec<_>>()
                    //     {
                    //         if engine.entity_by_id_untyped(&item.id, None).is_none() {
                    //             continue;
                    //         }
                    //         if item.rect().intersect(player_entity.rect()).is_empty() {
                    //             continue;
                    //         }
                    //         // otherwise pick up the item
                    //         engine.entities.remove(&item.id);
                    //
                    //         // mark user as having object
                    //         game_events_sender
                    //             .send(GameEvent::PlayerPickUp(
                    //                 player_entity
                    //                     .extract_ref::<PlayerEntity>()
                    //                     .unwrap()
                    //                     .player_id
                    //                     .clone(),
                    //                 item.item_type,
                    //                 item.count,
                    //             ))
                    //             .unwrap();
                    //         return;
                    //     }
                    // }
                }
                GameEvent::PlayerPickUp(_, _, _) => {
                    // update the inventory
                }
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
