use byteorder::{NetworkEndian, WriteBytesExt};
use serde_json;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use crate::server::store::SecureStore;

pub struct SecretsServer {
    pub store: Arc<Mutex<SecureStore>>,
}

impl SecretsServer {
    pub fn new() -> Self {
        let mut store = SecureStore::new();

        store
            .store_secret("DB_PASSWORD".to_string(), "super_secret_123".to_string())
            .expect("Failed to store DB password");
        store
            .store_secret("API_KEY".to_string(), "very_secret_key_456".to_string())
            .expect("Failed to store API key");

        SecretsServer {
            store: Arc::new(Mutex::new(store)),
        }
    }

    pub fn handle_client(self: &Self, mut stream: TcpStream) -> std::io::Result<()> {
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer)?;

        match serde_json::from_slice::<Vec<String>>(&buffer[..n]) {
            Ok(commands) if !commands.is_empty() && commands[0] == "get_env" => {
                let keys = &commands[1..];

                let mut response = HashMap::new();

                {
                    let store = self.store.lock().unwrap();

                    for key in keys {
                        if let Some(secret) = store.get_secret(key) {
                            response.insert(key.clone(), secret);
                        }
                    }
                }

                let response_json = serde_json::to_string(&response)?;
                let mut response_buffer = Vec::new();
                response_buffer.write_u32::<NetworkEndian>(response_json.len() as u32)?;
                response_buffer.extend_from_slice(response_json.as_bytes());

                stream.write_all(&response_buffer)?;
            }
            _ => {
                stream.write_all(&[0, 0, 0, 0])?; // Empty response
            }
        }

        Ok(())
    }

    pub fn run(self) -> std::io::Result<()> {
        let listener = TcpListener::bind("127.0.0.1:6000")?;
        println!("Server started successfully");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let server_clone = SecretsServer {
                        store: Arc::clone(&self.store),
                    };

                    std::thread::spawn(move || {
                        if let Err(e) = server_clone.handle_client(stream) {
                            eprintln!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }

        Ok(())
    }
}
