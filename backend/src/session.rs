use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConnectionSession {
    pub endpoint: String,
    pub auth_token: Option<String>,
    pub extra_headers: HashMap<String, String>,
}

#[derive(Default)]
pub struct SessionRegistry {
    sessions: HashMap<String, ConnectionSession>,
}

impl SessionRegistry {
    pub fn insert(&mut self, connection_id: String, session: ConnectionSession) {
        self.sessions.insert(connection_id, session);
    }

    pub fn remove(&mut self, connection_id: &str) {
        self.sessions.remove(connection_id);
    }

    pub fn get(&self, connection_id: &str) -> Option<&ConnectionSession> {
        self.sessions.get(connection_id)
    }
}
