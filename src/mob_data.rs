pub struct MobData {
    id: String,
    level_range: (u64, u64),
}

impl MobData {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct MobSpawnData {
    level_range: (u64, u64),
    health: u64,
    armor: u64,
    magic_resist: u64,
    knockback_resist: u64,
}
