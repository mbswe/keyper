use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn handle_decr(key: &str, store: &Arc<Mutex<HashMap<String, String>>>) -> String {
    let mut store_lock = store.lock().unwrap();
    let value = store_lock.entry(key.to_string()).or_insert("0".to_string());
    match value.parse::<i64>() {
        Ok(num) => {
            *value = (num - 1).to_string();
            value.clone()
        }
        Err(_) => "Error: Value is not an integer\n".to_string(),
    }
}
