use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn handle_keys(store: &Arc<Mutex<HashMap<String, String>>>) -> String {
    let store = store.lock().unwrap();
    let keys: Vec<String> = store.keys().cloned().collect();
    keys.join("\n") + "\n"
}
