#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter, Seek};
use std::path::Path;
use md5::{Md5, Digest};
use hex::encode as hex_encode;

pub mod crypto;

use crypto::{encrypt, decrypt, CryptoAlgorithm};
use std::str::FromStr;

/// 加密文件 - 适用于小到中等大小的文件
#[napi(js_name = "encryptFile")]
pub fn encrypt_file(algorithm: String, key: Buffer, input_path: String, output_path: String, env: Env) -> Result<Object> {
    let algo = CryptoAlgorithm::from_str(&algorithm)
        .map_err(|_| Error::from_reason("Invalid algorithm".to_string()))?;
    
    // 读取整个文件内容
    let mut file = match File::open(&input_path) {
        Ok(file) => file,
        Err(err) => return Err(Error::from_reason(format!("Failed to open input file: {}", err))),
    };
    
    // 获取文件大小
    let file_size = match file.metadata() {
        Ok(metadata) => metadata.len(),
        Err(err) => return Err(Error::from_reason(format!("Failed to get file metadata: {}", err))),
    };
    
    let mut data = Vec::new();
    if let Err(err) = file.read_to_end(&mut data) {
        return Err(Error::from_reason(format!("Failed to read input file: {}", err)));
    }
    
    // 使用一次性加密函数加密整个数据
    let encrypted = encrypt(algo, &key, &data)
        .map_err(|e| Error::from_reason(format!("Encryption error: {}", e)))?;
    
    // 写入加密数据到输出文件
    let mut output_file = match File::create(&output_path) {
        Ok(file) => file,
        Err(err) => return Err(Error::from_reason(format!("Failed to create output file: {}", err))),
    };
    
    if let Err(err) = output_file.write_all(&encrypted) {
        return Err(Error::from_reason(format!("Failed to write encrypted data: {}", err)));
    }
    
    // 计算KB单位的文件大小
    let file_size_kb = (file_size as f64) / 1024.0;
    
    // 创建并返回结果对象
    let mut result = env.create_object()?;
    result.set("fileSize", file_size_kb)?;
    
    Ok(result)
}

/// 解密文件 - 适用于小到中等大小的文件
#[napi(js_name = "decryptFile")]
pub fn decrypt_file(algorithm: String, key: Buffer, input_path: String, output_path: String, env: Env) -> Result<Object> {
    let algo = CryptoAlgorithm::from_str(&algorithm)
        .map_err(|_| Error::from_reason("Invalid algorithm".to_string()))?;
    
    // 读取整个加密文件
    let mut file = match File::open(&input_path) {
        Ok(file) => file,
        Err(err) => return Err(Error::from_reason(format!("Failed to open encrypted file: {}", err))),
    };
    
    // 获取加密文件大小
    let encrypted_file_size = match file.metadata() {
        Ok(metadata) => metadata.len(),
        Err(err) => return Err(Error::from_reason(format!("Failed to get file metadata: {}", err))),
    };
    
    let mut encrypted_data = Vec::new();
    if let Err(err) = file.read_to_end(&mut encrypted_data) {
        return Err(Error::from_reason(format!("Failed to read encrypted file: {}", err)));
    }
    
    // 使用一次性解密函数解密整个数据
    let decrypted = decrypt(algo, &key, &encrypted_data)
        .map_err(|e| Error::from_reason(format!("Decryption error: {}", e)))?;
    
    // 写入解密数据到输出文件
    let mut output_file = match File::create(&output_path) {
        Ok(file) => file,
        Err(err) => return Err(Error::from_reason(format!("Failed to create output file: {}", err))),
    };
    
    if let Err(err) = output_file.write_all(&decrypted) {
        return Err(Error::from_reason(format!("Failed to write decrypted data: {}", err)));
    }
    
    // 计算KB单位的文件大小
    let file_size_kb = (decrypted.len() as f64) / 1024.0;
    let encrypted_size_kb = (encrypted_file_size as f64) / 1024.0;
    
    // 创建并返回结果对象
    let mut result = env.create_object()?;
    result.set("fileSize", file_size_kb)?;
    result.set("encryptedSize", encrypted_size_kb)?;
    
    Ok(result)
}

/// 分片加密文件 - 用于超大文件，带有分片处理功能
#[napi(js_name = "chunkEncryptFile")]
pub fn chunk_encrypt_file(algorithm: String, key: Buffer, input_path: String, output_path: String, chunk_size_mb: u32, env: Env) -> Result<Object> {
    let algo = CryptoAlgorithm::from_str(&algorithm)
        .map_err(|_| Error::from_reason("Invalid algorithm".to_string()))?;
    
    // 默认使用10MB的块大小，也可以通过参数指定
    let chunk_size = (chunk_size_mb as usize) * 1024 * 1024;
    
    // 打开输入文件
    let input_file = match File::open(&input_path) {
        Ok(file) => file,
        Err(err) => return Err(Error::from_reason(format!("Failed to open input file: {}", err))),
    };
    
    let file_size = match input_file.metadata() {
        Ok(metadata) => metadata.len(),
        Err(err) => return Err(Error::from_reason(format!("Failed to get file metadata: {}", err))),
    };
    
    let mut reader = BufReader::with_capacity(chunk_size, input_file);
    
    // 创建输出文件
    let output_file = match File::create(&output_path) {
        Ok(file) => file,
        Err(err) => return Err(Error::from_reason(format!("Failed to create output file: {}", err))),
    };
    
    let mut writer = BufWriter::with_capacity(chunk_size, output_file);
    
    // 写入分片标记和元数据（文件头）
    let header = format!("CHUNKS:{}:{}:", file_size, chunk_size);
    if let Err(err) = writer.write_all(header.as_bytes()) {
        return Err(Error::from_reason(format!("Failed to write file header: {}", err)));
    }
    
    // 计算预期的总分片数，用于后续处理
    let _total_chunks = (file_size as f64 / chunk_size as f64).ceil() as u64;
    
    let mut buffer = vec![0u8; chunk_size];
    let mut chunk_index = 0;
    
    loop {
        let bytes_read = match reader.read(&mut buffer) {
            Ok(0) => break, // 读取完毕
            Ok(n) => n,
            Err(err) => return Err(Error::from_reason(format!("Error reading file chunk: {}", err))),
        };
        
        chunk_index += 1;
        
        // 只加密实际读取的数据
        let chunk_data = &buffer[..bytes_read];
        
        // 加密当前块
        let encrypted = match encrypt(algo.clone(), &key, chunk_data) {
            Ok(data) => data,
            Err(err) => return Err(Error::from_reason(format!("Chunk encryption error: {}", err))),
        };
        
        // 写入块大小和加密后的数据
        let size_header = format!("{}:", encrypted.len());
        if let Err(err) = writer.write_all(size_header.as_bytes()) {
            return Err(Error::from_reason(format!("Failed to write chunk size header: {}", err)));
        }
        
        if let Err(err) = writer.write_all(&encrypted) {
            return Err(Error::from_reason(format!("Failed to write encrypted chunk: {}", err)));
        }
        
        // 如果没读满buffer，说明文件已经读完了
        if bytes_read < chunk_size {
            break;
        }
    }
    
    // 确保所有数据都写入磁盘
    if let Err(err) = writer.flush() {
        return Err(Error::from_reason(format!("Failed to flush output file: {}", err)));
    }
    
    // 计算KB单位的大小
    let file_size_kb = (file_size as f64) / 1024.0;
    let chunk_size_kb = (chunk_size as f64) / 1024.0;
    
    // 创建并返回结果对象
    let mut result = env.create_object()?;
    result.set("totalChunks", chunk_index)?;
    result.set("fileSize", file_size_kb)?;
    result.set("chunkSize", chunk_size_kb)?;
    
    Ok(result)
}

/// 分片解密文件 - 用于超大文件，处理分片加密的文件
#[napi(js_name = "chunkDecryptFile")]
pub fn chunk_decrypt_file(algorithm: String, key: Buffer, input_path: String, output_path: String, env: Env) -> Result<Object> {
    let algo = CryptoAlgorithm::from_str(&algorithm)
        .map_err(|_| Error::from_reason("Invalid algorithm".to_string()))?;
    
    // 打开输入文件
    let mut input_file = match File::open(&input_path) {
        Ok(file) => file,
        Err(err) => return Err(Error::from_reason(format!("Failed to open input file: {}", err))),
    };
    
    // 创建输出文件
    let mut output_file = match File::create(&output_path) {
        Ok(file) => file,
        Err(err) => return Err(Error::from_reason(format!("Failed to create output file: {}", err))),
    };
    
    // 读取文件头以获取元数据
    let mut header = String::new();
    let mut buffer = [0u8; 1];
    
    loop {
        match input_file.read_exact(&mut buffer) {
            Ok(_) => {
                if buffer[0] == b':' {
                    break;
                }
                header.push(buffer[0] as char);
            },
            Err(err) => return Err(Error::from_reason(format!("Error reading header: {}", err))),
        }
    }
    
    if !header.starts_with("CHUNKS") {
        return Err(Error::from_reason("Invalid file format - not a chunked file".to_string()));
    }
    
    // 解析文件大小
    let mut original_size_str = String::new();
    loop {
        match input_file.read_exact(&mut buffer) {
            Ok(_) => {
                if buffer[0] == b':' {
                    break;
                }
                original_size_str.push(buffer[0] as char);
            },
            Err(err) => return Err(Error::from_reason(format!("Error reading file size: {}", err))),
        }
    }
    
    let original_size: u64 = match original_size_str.parse() {
        Ok(size) => size,
        Err(_) => return Err(Error::from_reason("Invalid file size in header".to_string())),
    };
    
    // 解析块大小
    let mut chunk_size_str = String::new();
    loop {
        match input_file.read_exact(&mut buffer) {
            Ok(_) => {
                if buffer[0] == b':' {
                    break;
                }
                chunk_size_str.push(buffer[0] as char);
            },
            Err(err) => return Err(Error::from_reason(format!("Error reading chunk size: {}", err))),
        }
    }
    
    let chunk_size: usize = match chunk_size_str.parse() {
        Ok(size) => size,
        Err(_) => return Err(Error::from_reason("Invalid chunk size in header".to_string())),
    };
    
    let mut total_bytes_written = 0;
    let mut chunk_index = 0;
    
    // 读取并解密每个块
    while total_bytes_written < original_size {
        // 读取块大小
        let mut chunk_enc_size_str = String::new();
        loop {
            match input_file.read_exact(&mut buffer) {
                Ok(_) => {
                    if buffer[0] == b':' {
                        break;
                    }
                    chunk_enc_size_str.push(buffer[0] as char);
                },
                Err(err) => return Err(Error::from_reason(format!("Error reading encrypted chunk size: {}", err))),
            }
        }
        
        let encrypted_chunk_size: usize = match chunk_enc_size_str.parse() {
            Ok(size) => size,
            Err(_) => return Err(Error::from_reason("Invalid encrypted chunk size".to_string())),
        };
        
        // 读取加密的块数据
        let mut encrypted_chunk = vec![0u8; encrypted_chunk_size];
        if let Err(err) = input_file.read_exact(&mut encrypted_chunk) {
            return Err(Error::from_reason(format!("Error reading encrypted chunk: {}", err)));
        }
        
        // 解密当前块
        let decrypted = match decrypt(algo.clone(), &key, &encrypted_chunk) {
            Ok(data) => data,
            Err(err) => return Err(Error::from_reason(format!("Chunk decryption error: {}", err))),
        };
        
        chunk_index += 1;
        
        // 写入解密后的数据
        if let Err(err) = output_file.write_all(&decrypted) {
            return Err(Error::from_reason(format!("Failed to write decrypted chunk: {}", err)));
        }
        
        total_bytes_written += decrypted.len() as u64;
        
        // 检查是否达到了原始文件大小
        if total_bytes_written >= original_size {
            break;
        }
    }
    
    // 计算KB单位的大小
    let original_size_kb = (original_size as f64) / 1024.0;
    let total_bytes_written_kb = (total_bytes_written as f64) / 1024.0;
    let chunk_size_kb = (chunk_size as f64) / 1024.0;
    
    // 创建并返回结果对象
    let mut result = env.create_object()?;
    result.set("totalChunks", chunk_index)?;
    result.set("totalBytesKB", total_bytes_written_kb)?;
    result.set("originalSizeKB", original_size_kb)?;
    result.set("chunkSizeKB", chunk_size_kb)?;
    
    Ok(result)
}

/// 单个分片的解密 - 用于视频实时播放场景
#[napi(js_name = "decryptSingleChunk")]
pub fn decrypt_single_chunk(algorithm: String, key: Buffer, input_path: String, chunk_index: u32) -> Result<Buffer> {
    let algo = CryptoAlgorithm::from_str(&algorithm)
        .map_err(|_| Error::from_reason("Invalid algorithm".to_string()))?;
    
    // 打开输入文件
    let mut input_file = match File::open(&input_path) {
        Ok(file) => file,
        Err(err) => return Err(Error::from_reason(format!("Failed to open input file: {}", err))),
    };
    
    // 读取文件头以获取元数据
    let mut header = String::new();
    let mut buffer = [0u8; 1];
    
    loop {
        match input_file.read_exact(&mut buffer) {
            Ok(_) => {
                if buffer[0] == b':' {
                    break;
                }
                header.push(buffer[0] as char);
            },
            Err(err) => return Err(Error::from_reason(format!("Error reading header: {}", err))),
        }
    }
    
    if !header.starts_with("CHUNKS") {
        return Err(Error::from_reason("Invalid file format - not a chunked file".to_string()));
    }
    
    // 解析文件大小
    let mut original_size_str = String::new();
    loop {
        match input_file.read_exact(&mut buffer) {
            Ok(_) => {
                if buffer[0] == b':' {
                    break;
                }
                original_size_str.push(buffer[0] as char);
            },
            Err(err) => return Err(Error::from_reason(format!("Error reading file size: {}", err))),
        }
    }
    
    // 解析块大小
    let mut chunk_size_str = String::new();
    loop {
        match input_file.read_exact(&mut buffer) {
            Ok(_) => {
                if buffer[0] == b':' {
                    break;
                }
                chunk_size_str.push(buffer[0] as char);
            },
            Err(err) => return Err(Error::from_reason(format!("Error reading chunk size: {}", err))),
        }
    }
    
    // 跳过前面的分块，找到目标分块
    let mut current_chunk = 0;
    while current_chunk < chunk_index {
        // 读取块大小
        let mut chunk_enc_size_str = String::new();
        loop {
            match input_file.read_exact(&mut buffer) {
                Ok(_) => {
                    if buffer[0] == b':' {
                        break;
                    }
                    chunk_enc_size_str.push(buffer[0] as char);
                },
                Err(err) => return Err(Error::from_reason(format!("Error reading encrypted chunk size: {}", err))),
            }
        }
        
        let encrypted_chunk_size: usize = match chunk_enc_size_str.parse() {
            Ok(size) => size,
            Err(_) => return Err(Error::from_reason("Invalid encrypted chunk size".to_string())),
        };
        
        // 跳过这个块
        if let Err(err) = input_file.seek(std::io::SeekFrom::Current(encrypted_chunk_size as i64)) {
            return Err(Error::from_reason(format!("Error seeking to next chunk: {}", err)));
        }
        
        current_chunk += 1;
    }
    
    // 读取目标块大小
    let mut chunk_enc_size_str = String::new();
    loop {
        match input_file.read_exact(&mut buffer) {
            Ok(_) => {
                if buffer[0] == b':' {
                    break;
                }
                chunk_enc_size_str.push(buffer[0] as char);
            },
            Err(err) => return Err(Error::from_reason(format!("Error reading target chunk size: {}", err))),
        }
    }
    
    let encrypted_chunk_size: usize = match chunk_enc_size_str.parse() {
        Ok(size) => size,
        Err(_) => return Err(Error::from_reason("Invalid encrypted chunk size".to_string())),
    };
    
    // 读取加密的块数据
    let mut encrypted_chunk = vec![0u8; encrypted_chunk_size];
    if let Err(err) = input_file.read_exact(&mut encrypted_chunk) {
        return Err(Error::from_reason(format!("Error reading encrypted chunk: {}", err)));
    }
    
    // 解密当前块
    let decrypted = match decrypt(algo, &key, &encrypted_chunk) {
        Ok(data) => data,
        Err(err) => return Err(Error::from_reason(format!("Chunk decryption error: {}", err))),
    };
    
    // 将解密后的数据返回为Buffer
    Ok(Buffer::from(decrypted))
}

/// 获取分片加密文件的元数据 - 用于视频播放前获取文件信息
#[napi(js_name = "getChunkedFileMetadata")]
pub fn get_chunked_file_metadata(input_path: String, env: Env) -> Result<Object> {
    // 打开输入文件
    let mut input_file = match File::open(&input_path) {
        Ok(file) => file,
        Err(err) => return Err(Error::from_reason(format!("Failed to open input file: {}", err))),
    };
    
    // 读取文件头以获取元数据
    let mut header = String::new();
    let mut buffer = [0u8; 1];
    
    loop {
        match input_file.read_exact(&mut buffer) {
            Ok(_) => {
                if buffer[0] == b':' {
                    break;
                }
                header.push(buffer[0] as char);
            },
            Err(err) => return Err(Error::from_reason(format!("Error reading header: {}", err))),
        }
    }
    
    if !header.starts_with("CHUNKS") {
        return Err(Error::from_reason("Invalid file format - not a chunked file".to_string()));
    }
    
    // 解析文件大小
    let mut original_size_str = String::new();
    loop {
        match input_file.read_exact(&mut buffer) {
            Ok(_) => {
                if buffer[0] == b':' {
                    break;
                }
                original_size_str.push(buffer[0] as char);
            },
            Err(err) => return Err(Error::from_reason(format!("Error reading file size: {}", err))),
        }
    }
    
    let original_size: u64 = match original_size_str.parse() {
        Ok(size) => size,
        Err(_) => return Err(Error::from_reason("Invalid file size in header".to_string())),
    };
    
    // 解析块大小
    let mut chunk_size_str = String::new();
    loop {
        match input_file.read_exact(&mut buffer) {
            Ok(_) => {
                if buffer[0] == b':' {
                    break;
                }
                chunk_size_str.push(buffer[0] as char);
            },
            Err(err) => return Err(Error::from_reason(format!("Error reading chunk size: {}", err))),
        }
    }
    
    let chunk_size: usize = match chunk_size_str.parse() {
        Ok(size) => size,
        Err(_) => return Err(Error::from_reason("Invalid chunk size in header".to_string())),
    };
    
    // 计算总块数
    let total_chunks = (original_size as f64 / chunk_size as f64).ceil() as u32;
    
    // 计算KB单位的大小
    let original_size_kb = (original_size as f64) / 1024.0;
    let chunk_size_kb = (chunk_size as f64) / 1024.0;
    
    // 创建并返回结果对象
    let mut result = env.create_object()?;
    result.set("totalChunks", total_chunks)?;
    result.set("fileSizeKB", original_size_kb)?;
    result.set("chunkSizeKB", chunk_size_kb)?;
    
    Ok(result)
}

/// 获取文件大小通用函数，用于测试文件操作
#[napi(js_name = "getFileSize")]
pub fn get_file_size(file_path: String) -> Result<f64> {
    let path = Path::new(&file_path);
    
    if !path.exists() {
        return Err(Error::from_reason(format!("File not found: {}", file_path)));
    }
    
    match std::fs::metadata(&file_path) {
        Ok(metadata) => {
            let size_kb = (metadata.len() as f64) / 1024.0;
            Ok(size_kb)
        },
        Err(err) => Err(Error::from_reason(format!("Failed to get file size: {}", err))),
    }
}

/// 计算文件的MD5哈希值
#[napi(js_name = "computeFileMd5")]
pub fn compute_file_md5(file_path: String) -> Result<String> {
    // 打开文件
    let file = match File::open(&file_path) {
        Ok(file) => file,
        Err(err) => return Err(Error::from_reason(format!("Failed to open file: {}", err))),
    };
    
    // 创建带缓冲的读取器，提高读取效率
    let mut reader = BufReader::with_capacity(8 * 1024 * 1024, file); // 8MB缓冲区
    
    // 创建MD5哈希计算器
    let mut hasher = Md5::new();
    
    // 分块读取文件并更新哈希值
    let mut buffer = [0u8; 1024 * 1024]; // 1MB 缓冲区
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break, // 文件读取完毕
            Ok(n) => {
                hasher.update(&buffer[..n]);
            },
            Err(err) => return Err(Error::from_reason(format!("Failed to read file: {}", err))),
        }
    }
    
    // 计算最终哈希值并转换为十六进制字符串
    let hash = hasher.finalize();
    let hex_hash = hex_encode(hash);
    
    Ok(hex_hash)
}
