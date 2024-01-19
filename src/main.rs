mod cmd;

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use serde::Deserialize;
use std::fs;
use log::{error, info, warn};
use simple_logger::SimpleLogger;
use std::time::Instant;

use cmd::get::handle_get;
use cmd::set::handle_set;
use cmd::check::handle_check;
use cmd::delete::handle_delete;
use cmd::keys::handle_keys;
use cmd::incr::handle_incr;
use cmd::decr::handle_decr;
use cmd::flush::handle_flush;
use cmd::mget::handle_mget;
use cmd::mset::handle_mset;
use cmd::info::handle_info;
use cmd::strlen::handle_strlen;

type Store = Arc<Mutex<HashMap<String, String>>>;

const JOURNAL_FILE: &str = "journal.log";

const VALID_JOURNAL_COMMANDS : [&str; 6] = [
    "SET",
    "MSET",
    "DELETE",
    "INCR",
    "DECR",
    "FLUSH"
];

#[derive(Deserialize)]
struct Config {
    server: ServerConfig,
}

#[derive(Deserialize)]
struct ServerConfig {
    bind: String,
    port: u16,
}

fn handle_client(mut stream: TcpStream, store: Store, journal: Arc<Mutex<File>>, start_time: Instant) {
    let mut buffer = [0; 1024];
    while match stream.read(&mut buffer) {
        Ok(size) => {
            let command = String::from_utf8_lossy(&buffer[..size]);
            let response = process_command(&command, &store, &journal, true, start_time);
            stream.write(response.as_bytes()).unwrap();
            true
        }
        Err(_) => {
            error!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(std::net::Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn process_command(
    command: &str,
    store: &Store,
    journal: &Arc<Mutex<File>>,
    write_to_journal: bool,
    start_time: Instant,
) -> String {
    let mut parts = command.trim().split_whitespace();
    let cmd = match parts.next() {
        Some(c) => c.to_uppercase(),
        None => return "Invalid command\n".to_string(),
    };

    let response = match cmd.as_str() {
        "GET" => {
            parts.next().and_then(|key| {
                if parts.next().is_none() { // Ensure no extra arguments
                    Some(handle_get(key, store))
                } else {
                    None
                }
            }).unwrap_or_else(|| "Invalid command\n".to_string())
        }
        "SET" => {
            let key = parts.next();
            let mut value_parts = Vec::new();
            let mut update = false;

            while let Some(part) = parts.next() {
                if part == "true" {
                    update = true;
                    break;
                }
                value_parts.push(part);
            }

            if let Some(key) = key {
                let value = value_parts.join(" ");
                handle_set(key.to_string(), value, update, store)
            } else {
                "Invalid command\n".to_string()
            }
        }
        "MGET" => {
            let keys: Vec<&str> = parts.collect();
            if !keys.is_empty() {
                handle_mget(&keys, store)
            } else {
                "Invalid command\n".to_string()
            }
        }
        "MSET" => {
            let mut key_value_pairs = Vec::new();
            let mut current_key = None;
            let mut current_value = String::new();
            let mut in_quotes = false;

            for part in parts {
                if in_quotes {
                    // Check if the current part ends with a quote
                    if part.ends_with('"') {
                        in_quotes = false;
                        current_value.push_str(&part[..part.len() - 1]); // Append part without the closing quote
                        if let Some(key) = current_key.take() {
                            key_value_pairs.push((key, current_value.clone()));
                        }
                        current_value.clear();
                    } else {
                        current_value.push_str(part);
                        current_value.push(' '); // Add space before the next part
                    }
                    continue;
                }

                if part.starts_with('"') {
                    in_quotes = true;
                    current_value.push_str(&part[1..]); // Append part without the opening quote
                    if part.ends_with('"') && part.len() > 1 { // Handle single-word quoted value
                        in_quotes = false;
                        let value = current_value.trim_end_matches('"').to_string();
                        current_value.clear();
                        if let Some(key) = current_key.take() {
                            key_value_pairs.push((key, value));
                        }
                    } else {
                        current_value.push(' '); // Add space before the next part
                    }
                    continue;
                }

                if let Some(key) = current_key.take() {
                    let value = if part.starts_with('"') && part.ends_with('"') && part.len() > 1 {
                        part[1..part.len() - 1].to_string()
                    } else {
                        part.to_string()
                    };
                    key_value_pairs.push((key, value));
                } else {
                    current_key = Some(part.to_string()); // Key
                }
            }

            if current_key.is_none() && !key_value_pairs.is_empty() && !in_quotes {
                handle_mset(key_value_pairs, store)
            } else {
                "Invalid command\n".to_string()
            }
        }

        "CHECK" | "DELETE" | "INCR" | "DECR" | "STRLEN" => {
            let args: Vec<&str> = parts.collect();
            if args.len() == 1 {
                match cmd.as_str() {
                    "CHECK" => handle_check(args[0], store),
                    "DELETE" => handle_delete(args[0], store),
                    "INCR" => handle_incr(args[0], store),
                    "DECR" => handle_decr(args[0], store),
                    "STRLEN" => handle_strlen(args[0], store),
                    _ => "Invalid command\n".to_string(),
                }
            } else {
                "Invalid command\n".to_string()
            }
        }
        "KEYS" | "FLUSH" | "INFO" => {
            if parts.next().is_none() {
                match cmd.as_str() {
                    "KEYS" => handle_keys(store),
                    "FLUSH" => handle_flush(store),
                    "INFO" => handle_info(store, start_time),
                    _ => "Invalid command\n".to_string(),
                }
            } else {
                "Invalid command\n".to_string()
            }
        }

        _ => "Invalid command\n".to_string(),
    };

    if write_to_journal && modifies_storage(&cmd) {
        let command_to_write = command.trim();
        let mut journal_lock = match journal.lock() {
            Ok(lock) => lock,
            Err(err) => {
                error!("Failed to lock journal for writing: {}", err);
                return "Error: Failed to write to journal\n".to_string();
            }
        };

        if let Err(err) = writeln!(journal_lock, "{}", command_to_write) {
            error!("Failed to write command to journal: {}", err);
            return "Error: Failed to write to journal\n".to_string();
        }
    }

    response
}

fn modifies_storage(command: &str) -> bool {
    VALID_JOURNAL_COMMANDS.contains(&command)
}

fn is_valid_journal_command(command: &str) -> bool {
    let parts: Vec<&str> = command.split_whitespace().collect();
    parts.get(0).map_or(false, |cmd| VALID_JOURNAL_COMMANDS.contains(cmd))
}

fn replay_journal(store: &Store, start_time: Instant) {
    let file = match File::open(JOURNAL_FILE) {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to open journal file: {}", e);
            return;
        },
    };

    let reader = BufReader::new(file);

    for (line_number, line) in reader.lines().enumerate() {
        let command = match line {
            Ok(cmd) => cmd,
            Err(err) => {
                error!("Error reading line {}: {}", line_number + 1, err);
                continue;
            },
        };

        if is_valid_journal_command(&command) {
            process_command(&command, store, &Arc::new(Mutex::new(File::open("/dev/null").unwrap())), false, start_time);
        } else {
            warn!("Invalid command at line {}: {}", line_number + 1, command);
        }
    }
}

fn main() {
    SimpleLogger::new().init().unwrap();
    let start_time = Instant::now();
    let store = Arc::new(Mutex::new(HashMap::new()));
    let journal = Arc::new(Mutex::new(OpenOptions::new().create(true).append(true).open(JOURNAL_FILE).unwrap()));
    let config_str = fs::read_to_string("config.toml")
        .expect("Failed to read config file");
    let config: Config = toml::from_str(&config_str)
        .expect("Failed to parse config file");

    replay_journal(&store, start_time);

    let listener = TcpListener::bind(format!("{}:{}", config.server.bind, config.server.port))
        .unwrap();
    info!("Server listening on {}:{}", config.server.bind, config.server.port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let store = Arc::clone(&store);
                let journal = Arc::clone(&journal);
                thread::spawn(move || {
                    handle_client(stream, store, journal, start_time);
                });
            }
            Err(e) => {
                error!("Connection failed: {}", e);
            }
        }
    }
}
