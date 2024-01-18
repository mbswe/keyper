use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn handle_set(key: String, value: String, update: bool, store: &Arc<Mutex<HashMap<String, String>>>) -> String {
    let mut store_lock = store.lock().unwrap();

    if update || !store_lock.contains_key(&key) {
        store_lock.insert(key.clone(), value.clone());
        "OK\n".to_string()
    } else {
        "Error: Key already exists\n".to_string()
    }
}
