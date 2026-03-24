use std::sync::Arc;

use dashmap::DashMap;

#[derive(Debug, Clone)]
pub struct Session {
    pub container_id: String,
    pub platform: String,
}

#[derive(Default, Clone)]
pub struct SessionStore(Arc<DashMap<String, Session>>);

impl SessionStore {
    pub fn insert(&self, id: String, session: Session) {
        self.0.insert(id, session);
    }

    pub fn get(&self, id: &str) -> Option<Session> {
        self.0.get(id).map(|s| s.clone())
    }

    pub fn remove(&self, id: &str) -> Option<Session> {
        self.0.remove(id).map(|(_, v)| v)
    }
}
