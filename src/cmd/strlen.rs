use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use unicode_segmentation::UnicodeSegmentation;

pub fn handle_strlen(key: &str, store: &Arc<Mutex<HashMap<String, String>>>) -> String {
    let store = store.lock().unwrap();
    store.get(key)
        .map(|value| {
            let trimmed_value = value.trim_matches('"');
            let grapheme_count = UnicodeSegmentation::graphemes(trimmed_value, true).count();
            grapheme_count.to_string()
        })
        .unwrap_or("0".to_string())
}
