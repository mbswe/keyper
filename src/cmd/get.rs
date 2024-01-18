use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn handle_get(key: &str, store: &Arc<Mutex<HashMap<String, String>>>) -> String {
    match store.lock().unwrap().get(key) {
        Some(value) => format!("{}\n", value),
        None => "Not found\n".to_string(),
    }
}
