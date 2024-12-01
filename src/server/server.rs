use byteorder::{NetworkEndian, WriteBytesExt};
use env_logger;
use log::{debug, error, info, trace, warn};
use serde_json;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use crate::server::store::SecureStore;
use crate::types::protected_secret::ProtectedSecret;

pub struct SecretsServer {
    pub store: Arc<Mutex<SecureStore>>,
}

impl SecretsServer {
    pub fn new() -> Self {
        let store = SecureStore::new();

        SecretsServer {
            store: Arc::new(Mutex::new(store)),
        }
    }

    pub fn handle_client(self: &Self, mut stream: TcpStream) -> std::io::Result<()> {
        info!(
            "Handling client connection on {}:{}",
            stream.peer_addr().unwrap().ip(),
            stream.peer_addr().unwrap().port()
        );

        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer)?;
        match serde_json::from_slice::<Vec<String>>(&buffer[..n]) {
            Ok(commands) if !commands.is_empty() && commands[0] == "get_env" => {
                info!("GET_ENV");
                let keys = &commands[1..];

                let mut response = HashMap::new();

                let store = self.store.lock();
                match store {
                    Ok(store) => {
                        for key in keys {
                            if let Some(secret) = Some(ProtectedSecret::new(store.get_secret(key)))
                            {
                                response.insert(key.clone(), secret);
                            }
                        }
                    }
                    Err(e) => error!("Error locking store: {}", e),
                }

                let response_json = serde_json::to_string(&response);
                match response_json {
                    Ok(response_json) => {
                        let mut response_buffer = Vec::new();
                        response_buffer.write_u32::<NetworkEndian>(response_json.len() as u32)?;
                        response_buffer.extend_from_slice(response_json.as_bytes());

                        let result = stream.write_all(&response_buffer);
                        match result {
                            Ok(_) => info!("Response sent successfully"),
                            Err(e) => error!("Error sending response: {}", e),
                        }
                    }
                    Err(e) => error!("Error serializing response: {}", e),
                }
            }

            Ok(commands) if !commands.is_empty() && commands[0].as_str() == "store_env" => {
                info!("STORE_ENV");
                if let Some(data) = commands.get(1) {
                    let secrets: HashMap<String, String> = match Some(data.as_str()) {
                        Some(s) => serde_json::from_str(s).map_err(|_| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid secrets data",
                            )
                        })?,
                        None => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid secrets data",
                            ))
                        }
                    };
                    let store = self.store.lock();
                    match store {
                        Ok(mut store) => {
                            for (key, value) in secrets {
                                store
                                    .store_secret(key, value)
                                    .expect("Failed to store secret");
                            }
                        }
                        Err(e) => error!("Error locking store: {}", e),
                    }

                    stream.write_all(&[0, 0, 0, 0])?;
                } else {
                    stream.write_all(&[0, 0, 0, 0])?; // Empty response
                }
            }
            _ => {
                stream.write_all(&[0, 0, 0, 0])?; // Empty response
                error!("Invalid command");
            }
        }

        Ok(())
    }

    pub fn run(self) -> std::io::Result<()> {
        env_logger::init();

        let listener = TcpListener::bind("127.0.0.1:6000")?;

        info!("Server started successfully on port 6000");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let server_clone = SecretsServer {
                        store: Arc::clone(&self.store),
                    };

                    std::thread::spawn(move || {
                        if let Err(e) = server_clone.handle_client(stream) {
                            error!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => error!("Connection failed: {}", e),
            }
        }

        Ok(())
    }
}
