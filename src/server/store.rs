use rand::RngCore;
use std::collections::HashMap;

// Secure memory block using Vec<u8> with explicit clearing
struct SecureMemoryBlock {
    memory: Vec<u8>,
}

impl SecureMemoryBlock {
    pub fn new(size: usize) -> Self {
        SecureMemoryBlock {
            memory: vec![0; size],
        }
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if data.len() > self.memory.len() {
            return Err("Data exceeds block size");
        }

        self.memory.fill(0);

        self.memory[..data.len()].copy_from_slice(data);
        Ok(())
    }

    pub fn read(&self) -> Vec<u8> {
        self.memory
            .iter()
            .cloned()
            .take_while(|&x| x != 0)
            .collect()
    }

    pub fn clear(&mut self) {
        self.memory.fill(0);
    }
}

impl Drop for SecureMemoryBlock {
    fn drop(&mut self) {
        self.clear();
    }
}

pub struct SecureStore {
    blocks: HashMap<String, SecureMemoryBlock>,
    pub keys: HashMap<String, Vec<u8>>,
}

impl SecureStore {
    pub fn new() -> Self {
        SecureStore {
            blocks: HashMap::new(),
            keys: HashMap::new(),
        }
    }

    pub fn store_secret(&mut self, key: String, value: String) -> Result<(), &'static str> {
        let mut encryption_key = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut encryption_key);

        let value_bytes = value.into_bytes();
        let encrypted_data = Self::encrypt(&value_bytes, &encryption_key);

        let block_size = encrypted_data.len() + 32; // Add padding
        let mut block = SecureMemoryBlock::new(block_size);

        block.write(&encrypted_data)?;

        self.blocks.insert(key.clone(), block);
        self.keys.insert(key, encryption_key);

        Ok(())
    }

    pub fn get_secret(&self, key: &str) -> Option<String> {
        self.blocks.get(key).and_then(|block| {
            self.keys.get(key).map(|encryption_key| {
                let encrypted_data = block.read();
                let decrypted_data = Self::decrypt(&encrypted_data, encryption_key);
                String::from_utf8(decrypted_data).unwrap_or_default()
            })
        })
    }

    pub fn encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
        data.iter()
            .zip(key.iter().cycle())
            .map(|(a, b)| a ^ b)
            .collect()
    }

    pub fn decrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
        Self::encrypt(data, key)
    }
}
