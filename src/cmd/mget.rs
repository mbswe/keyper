use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn handle_mget(keys: &[&str], store: &Arc<Mutex<HashMap<String, String>>>) -> String {
    let store = store.lock().unwrap();
    keys.iter()
        .map(|&key| store.get(key).cloned().unwrap_or_else(|| "Not found".to_string()))
        .collect::<Vec<String>>()
        .join("\n") + "\n"
}
