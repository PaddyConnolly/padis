use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Db {
    shared: Arc<Shared>,
}

struct Shared {
    state: Mutex<State>,
}

struct State {
    entries: HashMap<String, Entry>,
}

struct Entry {
    value: Bytes,
    expires_at: Option<Instant>,
}

impl Db {
    pub fn new() -> Db {
        Db {
            shared: Arc::new(Shared {
                state: Mutex::new(State {
                    entries: HashMap::new(),
                }),
            }),
        }
    }

    pub fn get(&self, key: &str) -> Option<Bytes> {
        let mut state = self.shared.state.lock().unwrap();

        let expired = state
            .entries
            .get(key)
            .map(|e| e.expires_at.is_some_and(|exp| Instant::now() > exp))
            .unwrap_or(false);

        if expired {
            state.entries.remove(key);
            return None;
        }

        state.entries.get(key).map(|e| e.value.clone())
    }

    pub fn set<K: Into<String>>(&self, key: K, value: Bytes, expiry: Option<Duration>) {
        let mut hm = match self.shared.state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let expires_at = expiry.map(|d| Instant::now() + d);
        hm.entries.insert(key.into(), Entry { value, expires_at });
    }

    pub fn del(&self, key: &str) -> bool {
        let mut hm = match self.shared.state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        hm.entries.remove(key).is_some()
    }

    pub fn keys(&self) -> Vec<String> {
        let mut hm = match self.shared.state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        hm.entries.retain(|_, entry| {
            entry
                .expires_at
                .map_or(true, |expiry| Instant::now() <= expiry)
        });
        return hm.entries.keys().cloned().collect();
    }
}
