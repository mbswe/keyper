use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn handle_flush(store: &Arc<Mutex<HashMap<String, String>>>) -> String {
    store.lock().unwrap().clear();
    "OK\n".to_string()
}
