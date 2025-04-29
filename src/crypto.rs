use aes::Aes256;
use block_modes::BlockMode;
use block_modes::Cbc;
use block_padding::Pkcs7;
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use chacha20poly1305::aead::{Aead, NewAead};
use rand::RngCore;
use std::str::FromStr;

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

#[derive(Clone, PartialEq)]
pub enum CryptoAlgorithm {
    Aes,
    Chacha20Poly1305,
}

impl FromStr for CryptoAlgorithm {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "aes" => Ok(CryptoAlgorithm::Aes),
            "chacha20poly1305" => Ok(CryptoAlgorithm::Chacha20Poly1305),
            _ => Err(()),
        }
    }
}

pub fn encrypt(algorithm: CryptoAlgorithm, key: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    match algorithm {
        CryptoAlgorithm::Aes => {
            if key.len() != 32 {
                return Err("AES key must be 32 bytes (256 bits)".to_string());
            }
            let mut iv = [0u8; 16];
            let mut rng = rand::rngs::ThreadRng::default();
            rng.fill_bytes(&mut iv);

            let cipher = Aes256Cbc::new_from_slices(key, &iv)
                .map_err(|e| format!("AES cipher init failed: {:?}", e))?;
            let ciphertext = cipher.encrypt_vec(data);

            let mut result = iv.to_vec();
            result.extend_from_slice(&ciphertext);
            Ok(result)
        }
        CryptoAlgorithm::Chacha20Poly1305 => {
            if key.len() != 32 {
                return Err("ChaCha20Poly1305 key must be 32 bytes (256 bits)".to_string());
            }

            let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
            let mut nonce = [0u8; 12];
            let mut rng = rand::rngs::ThreadRng::default();
            rng.fill_bytes(&mut nonce);

            let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), data)
                .map_err(|e| format!("ChaCha20Poly1305 encrypt failed: {:?}", e))?;

            let mut result = nonce.to_vec();
            result.extend_from_slice(&ciphertext);
            Ok(result)
        }
    }
}

pub fn decrypt(algorithm: CryptoAlgorithm, key: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    match algorithm {
        CryptoAlgorithm::Aes => {
            if key.len() != 32 {
                return Err("AES key must be 32 bytes (256 bits)".to_string());
            }
            if data.len() < 16 {
                return Err(format!("Invalid AES data: length {} is less than 16", data.len()));
            }
            
            let (iv, ciphertext) = data.split_at(16);
            
            // 确保密文长度是块大小的倍数
            if ciphertext.len() % 16 != 0 {
                return Err(format!("Invalid AES ciphertext length: {}. Must be multiple of 16", ciphertext.len()));
            }

            let cipher = Aes256Cbc::new_from_slices(key, iv)
                .map_err(|e| format!("AES cipher init failed: {:?}", e))?;

            cipher.decrypt_vec(ciphertext)
                .map_err(|e| format!("AES decrypt failed: {:?}, IV length: {}, ciphertext length: {}", 
                    e, iv.len(), ciphertext.len()))
        }
        CryptoAlgorithm::Chacha20Poly1305 => {
            if key.len() != 32 {
                return Err("ChaCha20Poly1305 key must be 32 bytes (256 bits)".to_string());
            }
            if data.len() < 12 {
                return Err("Invalid ChaCha20Poly1305 data".to_string());
            }
            let (nonce, ciphertext) = data.split_at(12);

            let cipher = ChaCha20Poly1305::new(Key::from_slice(key));

            cipher.decrypt(Nonce::from_slice(nonce), ciphertext)
                .map_err(|e| format!("ChaCha20Poly1305 decrypt failed: {:?}", e))
        }
    }
}