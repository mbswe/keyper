use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn handle_get(key: &str, store: &Arc<Mutex<HashMap<String, String>>>) -> String {
    let store = store.lock().unwrap();
    match store.get(key) {
        Some(value) => {
            let value = value.trim_matches('"');
            value.to_string()
        }
        None => "Key not found".to_string(),
    }
}
