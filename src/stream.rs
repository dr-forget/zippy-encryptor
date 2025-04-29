use aes::Aes256;
use block_modes::{BlockMode, Cbc};
use block_padding::Pkcs7;
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use chacha20poly1305::aead::{Aead, NewAead};
use rand::RngCore;

use crate::crypto::CryptoAlgorithm;

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

pub struct EncryptionStream {
    algorithm: CryptoAlgorithm,
    key: Vec<u8>,
    iv_or_nonce: Vec<u8>,
    buffer: Vec<u8>,
    block_size: usize,
    header_written: bool,
}

pub struct DecryptionStream {
    algorithm: CryptoAlgorithm,
    key: Vec<u8>,
    iv_or_nonce: Option<Vec<u8>>,
    buffer: Vec<u8>, // 用于收集加密数据的缓冲区
    block_size: usize,
}

impl EncryptionStream {
    pub fn new(algorithm: CryptoAlgorithm, key: &[u8]) -> Result<Self, String> {
        if key.len() != 32 {
            return Err(format!("Key must be 32 bytes (256 bits)"));
        }

        let (iv_or_nonce, block_size) = match algorithm {
            CryptoAlgorithm::Aes => {
                let mut iv = vec![0u8; 16];
                let mut rng = rand::rngs::ThreadRng::default();
                rng.fill_bytes(&mut iv);
                (iv, 16)
            },
            CryptoAlgorithm::Chacha20Poly1305 => {
                let mut nonce = vec![0u8; 12];
                let mut rng = rand::rngs::ThreadRng::default();
                rng.fill_bytes(&mut nonce);
                (nonce, 64) // ChaCha20Poly1305 doesn't have a block size, but we'll use 64 bytes for buffer size
            }
        };

        Ok(EncryptionStream {
            algorithm,
            key: key.to_vec(),
            iv_or_nonce,
            buffer: Vec::new(),
            block_size,
            header_written: false,
        })
    }

    pub fn process(&mut self, data: &[u8], output: &mut Vec<u8>) -> Result<(), String> {
        // Write IV/nonce as header if not already done
        if !self.header_written {
            output.extend_from_slice(&self.iv_or_nonce);
            self.header_written = true;
        }

        self.buffer.extend_from_slice(data);

        match self.algorithm {
            CryptoAlgorithm::Aes => {
                // 对于AES-CBC，我们需要保留一个块的大小作为填充，直到finalize
                let process_len = if self.buffer.len() > self.block_size {
                    self.buffer.len() - (self.buffer.len() % self.block_size)
                } else {
                    0
                };
                
                if process_len > 0 {
                    let cipher = Aes256Cbc::new_from_slices(&self.key, &self.iv_or_nonce)
                        .map_err(|e| format!("AES cipher init failed: {:?}", e))?;
                    
                    let chunk = &self.buffer[0..process_len];
                    let encrypted = cipher.encrypt_vec(chunk);
                    output.extend_from_slice(&encrypted);
                    
                    self.buffer.drain(0..process_len);
                }
            },
            CryptoAlgorithm::Chacha20Poly1305 => {
                if !self.buffer.is_empty() {
                    let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.key));
                    let encrypted = cipher.encrypt(Nonce::from_slice(&self.iv_or_nonce), &*self.buffer)
                        .map_err(|e| format!("ChaCha20Poly1305 encrypt failed: {:?}", e))?;
                    
                    output.extend_from_slice(&encrypted);
                    self.buffer.clear();
                }
            }
        }

        Ok(())
    }

    pub fn finalize(&mut self, output: &mut Vec<u8>) -> Result<(), String> {
        // Process any remaining data in the buffer
        if !self.buffer.is_empty() {
            match self.algorithm {
                CryptoAlgorithm::Aes => {
                    let cipher = Aes256Cbc::new_from_slices(&self.key, &self.iv_or_nonce)
                        .map_err(|e| format!("AES cipher init failed: {:?}", e))?;
                    
                    let encrypted = cipher.encrypt_vec(&self.buffer);
                    output.extend_from_slice(&encrypted);
                },
                CryptoAlgorithm::Chacha20Poly1305 => {
                    let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.key));
                    let encrypted = cipher.encrypt(Nonce::from_slice(&self.iv_or_nonce), &*self.buffer)
                        .map_err(|e| format!("ChaCha20Poly1305 encrypt failed: {:?}", e))?;
                    
                    output.extend_from_slice(&encrypted);
                }
            }
            self.buffer.clear();
        }
        
        Ok(())
    }
}

impl DecryptionStream {
    pub fn new(algorithm: CryptoAlgorithm, key: &[u8]) -> Result<Self, String> {
        if key.len() != 32 {
            return Err(format!("Key must be 32 bytes (256 bits)"));
        }

        let block_size = match algorithm {
            CryptoAlgorithm::Aes => 16,
            CryptoAlgorithm::Chacha20Poly1305 => 64,
        };

        Ok(DecryptionStream {
            algorithm,
            key: key.to_vec(),
            iv_or_nonce: None,
            buffer: Vec::new(),
            block_size,
        })
    }

    pub fn process(&mut self, data: &[u8], output: &mut Vec<u8>) -> Result<(), String> {
        // 添加新数据到缓冲区
        self.buffer.extend_from_slice(data);

        // 提取IV/nonce（如果尚未完成）
        if self.iv_or_nonce.is_none() {
            let header_size = match self.algorithm {
                CryptoAlgorithm::Aes => 16,
                CryptoAlgorithm::Chacha20Poly1305 => 12,
            };

            if self.buffer.len() < header_size {
                // 尚未接收到足够的数据来提取头部
                return Ok(());
            }

            self.iv_or_nonce = Some(self.buffer[..header_size].to_vec());
            self.buffer = self.buffer[header_size..].to_vec();
        }

        match self.algorithm {
            CryptoAlgorithm::Aes => {
                // 对于AES-CBC，流式解密是不安全的，因为它依赖于PKCS7填充
                // 这种填充只能在最后一个块中应用，因此我们需要等待所有数据
                // 在流式处理中，我们只是简单地收集所有数据，在finalize中一次性解密
            },
            CryptoAlgorithm::Chacha20Poly1305 => {
                // 尝试解密ChaCha20Poly1305加密的数据
                if !self.buffer.is_empty() && self.iv_or_nonce.is_some() {
                    let iv_or_nonce = self.iv_or_nonce.as_ref().unwrap();
                    let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.key));
                    
                    match cipher.decrypt(Nonce::from_slice(iv_or_nonce), &*self.buffer) {
                        Ok(decrypted) => {
                            output.extend_from_slice(&decrypted);
                            self.buffer.clear();
                        },
                        Err(_) => {
                            // 如果解密失败，可能是因为没有收到完整的消息，我们继续收集数据
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn finalize(&mut self, output: &mut Vec<u8>) -> Result<(), String> {
        // 确保我们有IV/nonce
        if self.iv_or_nonce.is_none() {
            return Err("No IV/nonce found in the encrypted data".to_string());
        }

        let iv_or_nonce = self.iv_or_nonce.as_ref().unwrap();

        match self.algorithm {
            CryptoAlgorithm::Aes => {
                // AES-CBC解密要求数据长度是块大小的倍数
                if self.buffer.len() % self.block_size != 0 {
                    return Err(format!(
                        "Invalid AES encrypted data length: {}. Must be multiple of block size {}",
                        self.buffer.len(), self.block_size
                    ));
                }

                // 一次性解密所有数据
                if !self.buffer.is_empty() {
                    let cipher = match Aes256Cbc::new_from_slices(&self.key, iv_or_nonce) {
                        Ok(c) => c,
                        Err(e) => return Err(format!("AES cipher init failed: {:?}", e)),
                    };
                    
                    let decrypted = match cipher.decrypt_vec(&self.buffer) {
                        Ok(d) => d,
                        Err(e) => {
                            // 提供更详细的错误诊断
                            return Err(format!("AES decrypt failed: {:?}. 这可能是由于加密和解密过程不匹配导致的。", e));
                        }
                    };
                    
                    output.extend_from_slice(&decrypted);
                }
            },
            CryptoAlgorithm::Chacha20Poly1305 => {
                // 如果还有未解密的数据，尝试解密
                if !self.buffer.is_empty() {
                    let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.key));
                    
                    let decrypted = match cipher.decrypt(Nonce::from_slice(iv_or_nonce), &*self.buffer) {
                        Ok(d) => d,
                        Err(e) => return Err(format!("ChaCha20Poly1305 decrypt failed: {:?}", e)),
                    };
                    
                    output.extend_from_slice(&decrypted);
                }
            }
        }
        
        // 清理缓冲区
        self.buffer.clear();
        
        Ok(())
    }
} 