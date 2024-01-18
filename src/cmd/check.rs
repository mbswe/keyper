use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn handle_check(key: &str, store: &Arc<Mutex<HashMap<String, String>>>) -> String {
    if store.lock().unwrap().contains_key(key) {
        "Exists\n".to_string()
    } else {
        "Not found\n".to_string()
    }
}
