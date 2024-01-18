use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn handle_delete(key: &str, store: &Arc<Mutex<HashMap<String, String>>>) -> String {
    if store.lock().unwrap().remove(key).is_some() {
        "OK\n".to_string()
    } else {
        "Key not found\n".to_string()
    }
}
