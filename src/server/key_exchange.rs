use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use num_bigint::{BigUint, RandBigInt};
use rand::Rng;
use sha2::{Digest, Sha256};

// Diffie-Hellman Key Exchange Struct
pub struct DHKeyExchange {
    private_key: BigUint,
    prime: BigUint,
    generator: BigUint,
}

impl DHKeyExchange {
    pub fn new() -> Self {
        let prime = BigUint::from_radix_be(
            "FFFFFFFFFFFFFFFFC90FDAA22168C234C4C6628B80DC1CD129024E088A67CC74020BBEA63B139B22514A08F3E1EF4E3E05A3BBB18C09E1A9BFC3B05A77C28BEE6A3A92D13B5D39B4C02FF0D0E98C8B4BE6C4B5433B7BB0C7B2A209B04CA00803111E5".as_bytes(),
            16
        ).unwrap();

        let generator = BigUint::from(2u32);

        let mut rng = rand::thread_rng();
        let private_key = rng.gen_biguint_below(&prime);

        DHKeyExchange {
            private_key,
            prime,
            generator,
        }
    }

    pub fn get_public_key(&self) -> BigUint {
        self.generator.modpow(&self.private_key, &self.prime)
    }

    pub fn compute_shared_secret(&self, other_public_key: &BigUint) -> Vec<u8> {
        let shared_secret = other_public_key.modpow(&self.private_key, &self.prime);

        let mut hasher = Sha256::new();
        hasher.update(shared_secret.to_bytes_be());
        hasher.finalize().to_vec()
    }

    pub fn encrypt(key: &[u8], data: &[u8]) -> Vec<u8> {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);

        let nonce = Nonce::from_slice(b"unique nonce");

        cipher.encrypt(nonce, data).expect("encryption failure!")
    }

    pub fn decrypt(key: &[u8], encrypted_data: &[u8]) -> Vec<u8> {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);

        let nonce = Nonce::from_slice(b"unique nonce");

        cipher
            .decrypt(nonce, encrypted_data)
            .expect("decryption failure!")
    }
}
