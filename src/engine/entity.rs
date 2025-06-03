use bevy::math::Rect;
use bevy::math::Vec2;

use super::mob::MobEntity;
use super::player::PlayerEntity;
use crate::MapData;

/// Inputs that may be applied to any entity.
#[derive(Default, PartialEq, Clone)]
pub struct EntityInput {
    pub jump: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub crouch: bool,
    pub attack: bool,
}

/// An entity that exists inside the engine.
pub trait Entity {
    fn id(&self) -> u64;
    fn position(&self) -> Vec2;
    fn size(&self) -> Vec2;
    fn step(&mut self, inputs: Option<&EntityInput>, map: &MapData) -> Self;

    fn rect(&self) -> Rect {
        let pos = self.position();
        let size = self.size();
        Rect::new(pos.x, pos.y, pos.x + size.x, pos.y + size.y)
    }
}

/// Enum to wrap all possible entity types
#[derive(Debug, Clone)]
pub enum EngineEntity {
    Player(PlayerEntity),
    Mob(MobEntity),
    // Item(Item),
}

impl Entity for EngineEntity {
    fn id(&self) -> u64 {
        match self {
            EngineEntity::Player(p) => p.id(),
            EngineEntity::Mob(m) => m.id(),
            // EngineEntity::Item(i) => i.id(),
        }
    }

    fn size(&self) -> Vec2 {
        match self {
            EngineEntity::Player(p) => p.size(),
            EngineEntity::Mob(m) => m.size(),
            // EngineEntity::Item(i) => i.position(),
        }
    }

    fn position(&self) -> Vec2 {
        match self {
            EngineEntity::Player(p) => p.position(),
            EngineEntity::Mob(m) => m.position(),
            // EngineEntity::Item(i) => i.position(),
        }
    }

    fn step(&mut self, inputs: Option<&EntityInput>, map: &MapData) -> Self {
        match self {
            EngineEntity::Player(p) => EngineEntity::Player(p.step(inputs, map)),
            EngineEntity::Mob(m) => EngineEntity::Mob(m.step(inputs, map)),
            // EngineEntity::Item(i) => i.step(inputs),
        }
    }
}
