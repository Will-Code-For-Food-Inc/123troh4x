use std::sync::Arc;

use dashmap::DashMap;

#[derive(Debug, Clone)]
pub struct Session {
    pub container_id: String,
    /// Stored for session listing and logging. Used by list_sessions (planned).
    #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn make_session(container_id: &str) -> Session {
        Session { container_id: container_id.into(), platform: "ds".into() }
    }

    #[test]
    fn insert_and_get() {
        let store = SessionStore::default();
        store.insert("abc".into(), make_session("ctr-1"));
        let s = store.get("abc").expect("should exist");
        assert_eq!(s.container_id, "ctr-1");
        assert_eq!(s.platform, "ds");
    }

    #[test]
    fn get_missing_returns_none() {
        let store = SessionStore::default();
        assert!(store.get("nonexistent").is_none());
    }

    #[test]
    fn remove_returns_session() {
        let store = SessionStore::default();
        store.insert("xyz".into(), make_session("ctr-2"));
        let s = store.remove("xyz").expect("should return session");
        assert_eq!(s.container_id, "ctr-2");
        assert!(store.get("xyz").is_none());
    }

    #[test]
    fn remove_missing_returns_none() {
        let store = SessionStore::default();
        assert!(store.remove("ghost").is_none());
    }

    #[test]
    fn concurrent_inserts_all_visible() {
        let store = SessionStore::default();
        let mut handles = vec![];

        for i in 0..50 {
            let s = store.clone();
            handles.push(thread::spawn(move || {
                s.insert(format!("session-{i}"), make_session(&format!("ctr-{i}")));
            }));
        }
        for h in handles {
            h.join().unwrap();
        }

        for i in 0..50 {
            assert!(
                store.get(&format!("session-{i}")).is_some(),
                "missing session-{i}"
            );
        }
    }

    #[test]
    fn concurrent_get_remove_no_panic() {
        let store = SessionStore::default();
        store.insert("shared".into(), make_session("ctr-shared"));

        let mut handles = vec![];
        for _ in 0..10 {
            let s = store.clone();
            handles.push(thread::spawn(move || {
                let _ = s.get("shared");
                let _ = s.remove("shared");
            }));
        }
        for h in handles {
            h.join().unwrap(); // must not panic
        }
    }
}
