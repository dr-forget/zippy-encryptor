# File Encryptor / 文件加密器

一个基于 Rust 实现的 Node.js 原生模块，用于文件加密和解密，支持高效处理大文件。

A Node.js native module (using Rust) for file encryption and decryption with large file support.

## 特性 / Features

- 使用 Rust 实现的快速、安全的加密/解密
- 支持 AES-256-CBC 和 ChaCha20Poly1305 算法
- 针对文件大小提供两种处理方式：标准和分块处理
- 针对大文件优化的分块处理方式，降低内存占用
- 基于 NAPI 的 Node.js 高性能绑定

- Fast, secure encryption/decryption implemented in Rust
- Support for AES-256-CBC and ChaCha20Poly1305 algorithms
- Two processing methods based on file size: standard and chunked
- Optimized chunked processing for large files to reduce memory usage
- High-performance Node.js bindings via NAPI

## 安装 / Installation

```bash
npm install
```

## 从源代码构建 / Building from source

```bash
npm run build
```

## 使用方法 / Usage

本库提供了四个主要方法用于文件加密和解密：

This library provides four main methods for file encryption and decryption:

1. `encryptFile` - 标准加密文件（适合较小文件）
2. `decryptFile` - 标准解密文件（适合较小文件）
3. `streamEncryptFile` - 分块加密大文件
4. `streamDecryptFile` - 分块解密大文件

### 文件大小和内存使用注意事项 / File Size and Memory Usage Considerations

- **小文件 (<100MB)**: 使用 `encryptFile` 和 `decryptFile` 方法，一次性加载整个文件
- **中等文件 (100MB-1GB)**: 建议使用 `streamEncryptFile` 和 `streamDecryptFile` 方法
- **大文件 (>1GB)**: 必须使用 `streamEncryptFile` 和 `streamDecryptFile` 方法，这些方法使用分块处理避免一次性加载整个文件

注意：即使使用流式方法，目前实现中解密仍需要一定的内存开销。超过 8GB 的文件可能会遇到内存限制。

- **Small files (<100MB)**: Use `encryptFile` and `decryptFile` methods, which load the entire file at once
- **Medium files (100MB-1GB)**: Recommended to use `streamEncryptFile` and `streamDecryptFile` methods
- **Large files (>1GB)**: Must use `streamEncryptFile` and `streamDecryptFile` methods, which process in chunks to avoid loading the entire file at once

Note: Even with streaming methods, the current implementation still requires some memory overhead for decryption. Files over 8GB may encounter memory limitations.

### 加密/解密小文件 / Encrypt/Decrypt Small Files

```javascript
const { encryptFile, decryptFile } = require("encryptor");
const crypto = require("crypto");
const fs = require("fs");

// 生成一个随机的256位密钥（32字节）
const key = Buffer.from(crypto.randomBytes(32));
// 可选：保存密钥以便之后复用
fs.writeFileSync("encryption_key.hex", key.toString("hex"));

// 加密文件
async function encryptSmallFile() {
  try {
    // 加密文件
    console.log("开始加密文件...");
    await encryptFile("aes", key, "./test.txt", "./test.txt.enc");
    console.log("加密完成");

    // 解密文件
    console.log("开始解密文件...");
    await decryptFile("aes", key, "./test.txt.enc", "./test.txt.dec");
    console.log("解密完成");
  } catch (error) {
    console.error("处理过程中发生错误:", error);
  }
}

encryptSmallFile();
```

### 加密/解密大文件 / Encrypt/Decrypt Large Files

```javascript
const { streamEncryptFile, streamDecryptFile } = require("encryptor");
const crypto = require("crypto");
const fs = require("fs");

// 使用之前保存的密钥
const key = Buffer.from(fs.readFileSync("encryption_key.hex", "utf8"), "hex");

// 分块处理大型文件
async function processLargeFile() {
  try {
    console.log("开始分块加密大文件...");
    await streamEncryptFile(
      "aes",
      key,
      "./large_video.mp4",
      "./large_video.mp4.enc"
    );
    console.log("加密完成");

    console.log("开始分块解密大文件...");
    await streamDecryptFile(
      "aes",
      key,
      "./large_video.mp4.enc",
      "./large_video.mp4.dec"
    );
    console.log("解密完成");
  } catch (error) {
    console.error("处理过程中发生错误:", error);
  }
}

processLargeFile();
```

## 完整演示 / Complete Demo

本库附带了一个完整的演示脚本 `test.js`，展示了如何：

- 生成和管理加密密钥
- 加密和解密小文件
- 分块加密和解密大文件
- 验证加密/解密过程的正确性

运行演示:

```bash
node test.js
```

## API 参考 / API Reference

### `encryptFile(algorithm, key, input_path, output_path)`

标准加密文件。适合小到中等大小的文件。

Standard file encryption. Suitable for small to medium-sized files.

- `algorithm`: 字符串，'aes'或'chacha20poly1305'
- `key`: Buffer，32 字节（256 位）
- `input_path`: 字符串，输入文件的路径
- `output_path`: 字符串，加密后输出文件的路径
- 返回: Promise<void>，操作完成时解析

- `algorithm`: String, either 'aes' or 'chacha20poly1305'
- `key`: Buffer, 32 bytes (256 bits)
- `input_path`: String, path to the input file
- `output_path`: String, path for the encrypted output file
- Returns: Promise<void> that resolves when the operation is complete

### `decryptFile(algorithm, key, input_path, output_path)`

标准解密文件。适合小到中等大小的文件。

Standard file decryption. Suitable for small to medium-sized files.

- `algorithm`: 字符串，'aes'或'chacha20poly1305'
- `key`: Buffer，32 字节（256 位）
- `input_path`: 字符串，加密文件的路径
- `output_path`: 字符串，解密后输出文件的路径
- 返回: Promise<void>，操作完成时解析

- `algorithm`: String, either 'aes' or 'chacha20poly1305'
- `key`: Buffer, 32 bytes (256 bits)
- `input_path`: String, path to the encrypted file
- `output_path`: String, path for the decrypted output file
- Returns: Promise<void> that resolves when the operation is complete

### `streamEncryptFile(algorithm, key, input_path, output_path)`

分块加密文件。适合大型文件，使用较少内存。

Chunked file encryption. Suitable for large files with minimal memory usage.

- `algorithm`: 字符串，'aes'或'chacha20poly1305'
- `key`: Buffer，32 字节（256 位）
- `input_path`: 字符串，输入文件的路径
- `output_path`: 字符串，加密后输出文件的路径
- 返回: Promise<void>，操作完成时解析

- `algorithm`: String, either 'aes' or 'chacha20poly1305'
- `key`: Buffer, 32 bytes (256 bits)
- `input_path`: String, path to the input file
- `output_path`: String, path for the encrypted output file
- Returns: Promise<void> that resolves when the operation is complete

### `streamDecryptFile(algorithm, key, input_path, output_path)`

分块解密文件。适合大型文件，使用较少内存。

Chunked file decryption. Suitable for large files with minimal memory usage.

- `algorithm`: 字符串，'aes'或'chacha20poly1305'
- `key`: Buffer，32 字节（256 位）
- `input_path`: 字符串，加密文件的路径
- `output_path`: 字符串，解密后输出文件的路径
- 返回: Promise<void>，操作完成时解析

- `algorithm`: String, either 'aes' or 'chacha20poly1305'
- `key`: Buffer, 32 bytes (256 bits)
- `input_path`: String, path to the encrypted file
- `output_path`: String, path for the decrypted output file
- Returns: Promise<void> that resolves when the operation is complete

## 支持的算法 / Supported Algorithms

- `aes`: AES-256-CBC，使用 PKCS7 填充
- `chacha20poly1305`: ChaCha20-Poly1305 AEAD 认证加密

- `aes`: AES-256-CBC with PKCS7 padding
- `chacha20poly1305`: ChaCha20-Poly1305 AEAD (Authenticated Encryption with Associated Data)

## 注意事项 / Notes

- 对于超过 8GB 的文件，您可能需要进一步定制此库，或考虑拆分大文件
- AES-CBC 模式适合一般用途，而 ChaCha20Poly1305 提供更强的安全性（包括消息认证）
- 确保安全存储密钥，密钥一旦丢失，数据将无法恢复
- 此库设计用于本地文件加密，不建议用于网络传输场景

- For files larger than 8GB, you may need to further customize this library or consider splitting large files
- AES-CBC mode is suitable for general purposes, while ChaCha20Poly1305 provides stronger security (including message authentication)
- Ensure keys are stored securely - if a key is lost, data cannot be recovered
- This library is designed for local file encryption and is not recommended for network transmission scenarios

## 许可证 / License

ISC

## 超大文件处理 / Extremely Large Files

对于超过 8GB 的大文件（如 20GB 视频），内置的`streamEncryptFile`和`streamDecryptFile`函数可能会遇到内存限制。此时，可以使用分片处理方法：

For files larger than 8GB (such as 20GB videos), the built-in `streamEncryptFile` and `streamDecryptFile` functions may encounter memory limitations. In such cases, you can use the chunked processing method:

### 基于 JavaScript 的分片处理 / JavaScript-based Chunking

```javascript
const { chunkedEncryptFile, chunkedDecryptFile } = require("./split_encrypt");
const crypto = require("crypto");
const fs = require("fs");

// 使用之前保存的密钥
const key = Buffer.from(fs.readFileSync("encryption_key.hex", "utf8"), "hex");

async function processUltraLargeFile() {
  try {
    console.log("开始分片加密超大文件...");
    await chunkedEncryptFile(
      "aes",
      key,
      "./large_video.mp4",
      "./large_video.mp4.enc"
    );
    console.log("加密完成");

    console.log("开始分片解密超大文件...");
    await chunkedDecryptFile(
      "aes",
      key,
      "./large_video.mp4.enc",
      "./large_video.mp4.dec"
    );
    console.log("解密完成");
  } catch (error) {
    console.error("处理过程中发生错误:", error);
  }
}

processUltraLargeFile();
```

分片处理方法会将大文件分成多个小于 4GB 的块，分别加密/解密后再合并，从而绕过内存限制。这种方法适用于任何大小的文件，特别是超过 8GB 的超大文件。

The chunked processing method divides large files into smaller chunks (less than 4GB each), encrypts/decrypts them separately, and then merges them back together. This bypasses memory limitations and is suitable for files of any size, especially those larger than 8GB.

### 基于 Rust 的分片处理（推荐） / Rust-based Chunking (Recommended)

针对超大文件，我们现在提供了直接在 Rust 中实现的分片处理功能，性能更高，内存管理更高效：

For extremely large files, we now provide chunking functionality implemented directly in Rust, which offers better performance and more efficient memory management:

```javascript
const { chunkEncryptFile, chunkDecryptFile } = require("encryptor");
const crypto = require("crypto");
const fs = require("fs");

// 使用之前保存的密钥
const key = Buffer.from(fs.readFileSync("encryption_key.hex", "utf8"), "hex");

async function processUltraLargeFile() {
  try {
    // 使用 10MB 块大小进行分片加密
    console.log("开始分片加密超大文件...");
    await chunkEncryptFile(
      "aes",
      key,
      "./large_video.mp4",
      "./large_video.mp4.enc",
      10 // 分块大小（MB）
    );
    console.log("加密完成");

    // 分片解密（自动检测块大小）
    console.log("开始分片解密超大文件...");
    await chunkDecryptFile(
      "aes",
      key,
      "./large_video.mp4.enc",
      "./large_video.mp4.dec"
    );
    console.log("解密完成");
  } catch (error) {
    console.error("处理过程中发生错误:", error);
  }
}

processUltraLargeFile();
```

Rust 实现的分片功能优点：

- 内存使用效率更高 - 只保留当前处理的数据块
- 更快的处理速度 - 利用 Rust 的高性能
- 可以处理任意大小的文件，适合 20GB 或更大的视频文件
- 支持自定义块大小，可根据可用内存调整

The advantages of the Rust-based chunking implementation:

- More efficient memory usage - only keeps the current chunk in memory
- Faster processing speed - leverages Rust's high performance
- Can handle files of any size, suitable for 20GB or larger video files
- Supports custom chunk sizes that can be adjusted based on available memory

### 使用 Rust 分块进行文件测试示例 / Example for Testing Files with Rust Chunking

您可以使用提供的测试脚本来验证分块功能：

You can use the provided test script to verify the chunking functionality:

```bash
# 使用 10MB 块大小测试 video.mp4 文件
node chunk_test.js
```

这个脚本会对文件进行分块加密和解密，并验证原始文件和解密后文件的完整性。

This script will perform chunked encryption and decryption on a file and verify the integrity of the original and decrypted files.

### 使用独立脚本处理超大文件 / Using Standalone Script for Extremely Large Files

您也可以直接使用`split_encrypt.js`脚本处理超大文件：

You can also directly use the `split_encrypt.js` script to process extremely large files:

```bash
# 加密和解密文件
node split_encrypt.js /path/to/large_video.mp4

# 只加密
node split_encrypt.js /path/to/large_video.mp4 encrypt

# 只解密
node split_encrypt.js /path/to/large_video.mp4 decrypt
```

这个脚本会自动处理超大文件的分片加密和解密，并管理临时文件。

This script automatically handles the chunked encryption and decryption of extremely large files and manages temporary files.
