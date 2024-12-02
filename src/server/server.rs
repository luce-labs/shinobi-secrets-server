use byteorder::{NetworkEndian, WriteBytesExt};
use daemonize::Daemonize;
use env_logger;
use log::{error, info};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client,
};
use serde::Serialize;
use serde_json::{self, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use crate::server::store::SecureStore;
use crate::types::protected_secret::ProtectedSecret;

#[derive(Clone)]
pub struct SecretsServer {
    pub store: Arc<Mutex<SecureStore>>,
    pub client: Client,
    pub base_url: String,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct GetKeysInput {
    pub project_name: String,
    pub token: String,
}

impl SecretsServer {
    pub fn new(base_url: String, token: String) -> Self {
        let store = SecureStore::new();
        let client = Client::new();

        SecretsServer {
            store: Arc::new(Mutex::new(store)),
            client,
            base_url,
            token,
        }
    }

    pub async fn build_project(
        &self,
        input: GetKeysInput,
    ) -> Result<serde_json::Value, Box<dyn Error>> {
        let url = format!("{}/projects/getkeys", self.base_url);

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.token))?,
        );

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&input)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::OK {
            let project: Value = response.json().await?;
            info!("{:?}", project);
            Ok(project)
        } else {
            let error_msg: Value = response.json().await?;
            Err(format!("{:?}", error_msg).into())
        }
    }

    pub async fn handle_client(self: &Self, mut stream: TcpStream) -> std::io::Result<()> {
        info!(
            "Handling client connection on {}:{}",
            stream.peer_addr().unwrap().ip(),
            stream.peer_addr().unwrap().port()
        );

        let mut buffer = [0; 1024];
        let n = match stream.read(&mut buffer) {
            Ok(n) if n > 0 => n,
            Ok(_) => {
                error!("Received empty data from client");
                return Ok(()); // Exit gracefully
            }
            Err(e) => {
                error!("Error reading from client: {}", e);
                return Err(e);
            }
        };
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

    pub fn get_keys(
        &self,
        project_name: String,
        token: String,
    ) -> Result<HashMap<String, String>, String> {
        let mut keys = HashMap::new();

        let store = self.store.lock();
        match store {
            Ok(store) => {
                let key = format!("{}_{}", project_name, token);
                if let Some(secret) = store.get_secret(&key) {
                    keys.insert("keys".to_string(), secret);
                }
            }
            Err(e) => return Err(format!("Error locking store: {}", e)),
        }

        Ok(keys)
    }

    pub async fn run(self, input: GetKeysInput) -> std::io::Result<()> {
        env_logger::init();

        let listener = TcpListener::bind("127.0.0.1:6000")?;
        listener.set_nonblocking(true)?;
        info!("Server started successfully on port 6000");

        let server = Arc::new(self);

        match server.build_project(input).await {
            Ok(project) => {
                info!("Project built successfully: {:?}", project);

                // Extract and store the secrets
                if let Some(keys) = project.get("keys").and_then(|keys| keys.as_object()) {
                    let store = server.store.lock();
                    if let Ok(mut store) = store {
                        for (key, value) in keys {
                            if let Some(value_str) = value.as_str() {
                                store
                                    .store_secret(key.clone(), value_str.to_string())
                                    .unwrap_or_else(|e| {
                                        error!("Failed to store key '{}': {}", key, e);
                                    });
                            }
                        }
                    } else {
                        error!("Failed to lock store for storing secrets");
                    }
                } else {
                    error!("No valid keys found in the project response");
                }
            }
            Err(e) => {
                error!("Failed to build project: {}", e);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ));
            }
        }

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let server_clone = Arc::clone(&server);
                    tokio::spawn(async move {
                        if let Err(e) = server_clone.handle_client(stream).await {
                            error!("Error handling client: {}", e);
                        }
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // This error is expected in non-blocking mode
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
                Err(e) => {
                    error!("Connection failed: {}", e);
                }
            }
        }

        Ok(())
    }
}
