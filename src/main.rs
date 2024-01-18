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

fn handle_client(mut stream: TcpStream, store: Arc<Mutex<HashMap<String, String>>>, journal: Arc<Mutex<File>>) {
    let mut buffer = [0; 1024];
    while match stream.read(&mut buffer) {
        Ok(size) => {
            let command = String::from_utf8_lossy(&buffer[..size]);
            let response = process_command(&command, &store, &journal, true);
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

fn process_command(command: &str, store: &Arc<Mutex<HashMap<String, String>>>, journal: &Arc<Mutex<File>>, write_to_journal: bool) -> String {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    if parts.is_empty() {
        return "Invalid command\n".to_string();
    }

    let response = match parts[0] {
        "GET" if parts.len() == 2 => {
            handle_get(parts[1], store)
        }
        "SET" if parts.len() >= 3 => {
            let update = parts.len() == 4 && parts[3] == "true";
            handle_set(parts[1].to_string(), parts[2].to_string(), update, store)
        }
        "MGET" if parts.len() > 1 => {
            handle_mget(&parts[1..], store)
        }
        "MSET" if parts.len() > 2 && parts.len() % 2 == 1 => {
            let pairs = parts[1..].chunks(2).collect::<Vec<_>>();
            handle_mset(&pairs.iter().map(|chunk| (chunk[0], chunk[1])).collect::<Vec<_>>(), store)
        }
        "CHECK" if parts.len() == 2 => {
            handle_check(parts[1], store)
        }
        "DELETE" if parts.len() == 2 => {
            handle_delete(parts[1], store)
        }
        "KEYS" if parts.len() == 1 => {
            handle_keys(store)
        }
        "INCR" if parts.len() == 2 => {
            handle_incr(parts[1], store)
        }
        "DECR" if parts.len() == 2 => {
            handle_decr(parts[1], store)
        }
        "FLUSH" if parts.len() == 1 => {
            handle_flush(store)
        }
        _ => "Invalid command\n".to_string(),
    };

    if write_to_journal && modifies_storage(parts[0]) {
        writeln!(journal.lock().unwrap(), "{}", command.trim()).unwrap();
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

fn replay_journal(store: &Arc<Mutex<HashMap<String, String>>>) {
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
            process_command(&command, store, &Arc::new(Mutex::new(File::open("/dev/null").unwrap())), false);
        } else {
            warn!("Invalid command at line {}: {}", line_number + 1, command);
        }
    }
}

fn main() {
    SimpleLogger::new().init().unwrap();
    let store = Arc::new(Mutex::new(HashMap::new()));
    let journal = Arc::new(Mutex::new(OpenOptions::new().create(true).append(true).open(JOURNAL_FILE).unwrap()));
    let config_str = fs::read_to_string("config.toml")
        .expect("Failed to read config file");
    let config: Config = toml::from_str(&config_str)
        .expect("Failed to parse config file");

    replay_journal(&store);

    let listener = TcpListener::bind(format!("{}:{}", config.server.bind, config.server.port))
        .unwrap();
    info!("Server listening on {}:{}", config.server.bind, config.server.port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let store = Arc::clone(&store);
                let journal = Arc::clone(&journal);
                thread::spawn(move || {
                    handle_client(stream, store, journal);
                });
            }
            Err(e) => {
                error!("Connection failed: {}", e);
            }
        }
    }
}
