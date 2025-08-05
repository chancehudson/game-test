use std::sync::Arc;

use bevy::prelude::*;

#[derive(Resource)]
pub struct Database(pub Arc<redb::Database>);

impl Default for Database {
    fn default() -> Self {
        let backend = redb::backends::InMemoryBackend::new();
        let database = redb::Database::builder()
            .create_with_backend(backend)
            .unwrap();
        Database(db::init(database).unwrap())
    }
}

pub struct DatabasePlugin;

impl Plugin for DatabasePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Database>();
    }
}
