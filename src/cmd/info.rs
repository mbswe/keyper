use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Instant};

pub fn handle_info(store: &Arc<Mutex<HashMap<String, String>>>, start_time: Instant) -> String {
    let store = store.lock().unwrap();
    let uptime = start_time.elapsed();

    format!(
        "Number of keys: {}\nUptime: {} seconds\n",
        store.len(),
        uptime.as_secs()
    )
}