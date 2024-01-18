use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn handle_mset(pairs: &[(&str, &str)], store: &Arc<Mutex<HashMap<String, String>>>) -> String {
    let mut store = store.lock().unwrap();
    for &(key, value) in pairs {
        store.insert(key.to_string(), value.to_string());
    }
    "OK\n".to_string()
}
