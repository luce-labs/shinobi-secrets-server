use libc::{c_void, mmap, mprotect, munmap, PROT_NONE, PROT_READ, PROT_WRITE};
use log::warn;
use page_size;
use rand::RngCore;
use std::collections::HashMap;
use std::ptr;
use std::slice;

struct SecureMemoryBlock {
    ptr: *mut u8,
    size: usize,
}

impl SecureMemoryBlock {
    pub fn new(size: usize) -> Result<Self, std::io::Error> {
        let page_size = page_size::get();
        let aligned_size = ((size + page_size - 1) / page_size) * page_size;

        let ptr = unsafe {
            mmap(
                ptr::null_mut(),
                aligned_size,
                PROT_READ | PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };

        if ptr == libc::MAP_FAILED {
            return Err(std::io::Error::last_os_error());
        }

        Ok(SecureMemoryBlock {
            ptr: ptr as *mut u8,
            size: aligned_size,
        })
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if data.len() > self.size {
            return Err("Data exceeds block size");
        }

        self.clear();

        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), self.ptr, data.len());
        }

        Ok(())
    }

    pub fn read(&self) -> Vec<u8> {
        unsafe {
            slice::from_raw_parts(self.ptr, self.size)
                .to_vec()
                .into_iter()
                .take_while(|&x| x != 0)
                .collect()
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            ptr::write_bytes(self.ptr, 0, self.size);
        }
    }

    pub fn lock(&self) -> Result<(), std::io::Error> {
        let result = unsafe { mprotect(self.ptr as *mut c_void, self.size, PROT_NONE) };

        if result == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

impl Drop for SecureMemoryBlock {
    fn drop(&mut self) {
        self.clear();

        unsafe {
            munmap(self.ptr as *mut c_void, self.size);
        }
    }
}

unsafe impl Send for SecureMemoryBlock {}
unsafe impl Sync for SecureMemoryBlock {}

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

    pub fn store_secret(&mut self, key: String, value: String) -> Result<(), String> {
        let mut encryption_key = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut encryption_key);

        let value_bytes = value.into_bytes();
        let encrypted_data = Self::encrypt(&value_bytes, &encryption_key);
        let block_size = encrypted_data.len() + 32;

        let mut block = SecureMemoryBlock::new(block_size).map_err(|e| e.to_string())?;

        block.write(&encrypted_data).map_err(|e| e.to_string())?;

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
