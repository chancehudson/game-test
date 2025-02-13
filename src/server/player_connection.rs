use std::collections::HashMap;

pub struct PlayerConnection {
    // socket id keyed to player
    socket_player_map: HashMap<String, String>,
    // player id keyed to socket
    player_socket_map: HashMap<String, String>,
}

impl PlayerConnection {
    pub fn new() -> Self {
        Self {
            socket_player_map: HashMap::new(),
            player_socket_map: HashMap::new(),
        }
    }

    pub async fn player_by_socket_id(&self, socket_id: &str) -> Option<String> {
        if let Some(player_id) = self.socket_player_map.get(socket_id) {
            if let Some(socket_id_internal) = self.player_socket_map.get(player_id) {
                if socket_id == socket_id_internal {
                    return Some(player_id.clone());
                }
            }
        }
        None
    }

    pub async fn socket_by_player_id(&self, player_id: &str) -> Option<String> {
        self.player_socket_map.get(player_id).cloned()
    }

    pub async fn register_player(&mut self, socket_id: String, player_id: String) {
        self.socket_player_map
            .insert(socket_id.clone(), player_id.clone());
        self.player_socket_map.insert(player_id, socket_id);
    }
}
