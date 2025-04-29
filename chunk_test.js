const {
  chunkEncryptFile,
  chunkDecryptFile,
  getFileSize,
  computeFileMd5,
} = require("./");
const crypto = require("crypto");
const fs = require("fs");
const path = require("path");

// 生成或读取一个密钥
let key;
const keyFile = path.join(__dirname, "encryption_key.hex");

try {
  if (fs.existsSync(keyFile)) {
    const keyHex = fs.readFileSync(keyFile, "utf8").trim();
    key = Buffer.from(keyHex, "hex");
    console.log("使用已存在的密钥");
  } else {
    // 生成一个随机的256位密钥（32字节）
    key = crypto.randomBytes(32);
    fs.writeFileSync(keyFile, key.toString("hex"));
    console.log("生成了新的密钥并保存");
  }
} catch (error) {
  console.error("处理密钥时出错:", error);
  process.exit(1);
}

// 加密算法（可选 'aes' 或 'chacha20poly1305'）
const algorithm = "chacha20poly1305";

// 测试文件路径
const testFile = {
  original: path.join(__dirname, "video.mp4"),
  encrypted: path.join(__dirname, "video.mp4.chunk.enc"),
  decrypted: path.join(__dirname, "video.mp4.chunk.dec"),
};

// 分块大小（MB）
const chunkSizeMB = 10;

// 验证文件是否存在
function ensureFileExists(filePath, errorMessage) {
  if (!fs.existsSync(filePath)) {
    console.error(errorMessage);
    process.exit(1);
  }
}

// 删除文件（如果存在）
function removeFileIfExists(filePath) {
  if (fs.existsSync(filePath)) {
    fs.unlinkSync(filePath);
    console.log(`删除了文件: ${filePath}`);
  }
}

// 清理旧文件
function cleanupFiles() {
  removeFileIfExists(testFile.encrypted);
  removeFileIfExists(testFile.decrypted);
}

// 比较两个文件的大小
async function compareFileSizes(file1, file2) {
  try {
    const size1 = await getFileSize(file1);
    const size2 = await getFileSize(file2);

    console.log(`原始文件大小: ${size1} 字节`);
    console.log(`解密后文件大小: ${size2} 字节`);

    if (size1 === size2) {
      console.log("✅ 验证成功: 文件大小匹配");
      return true;
    } else {
      console.error(
        `❌ 验证失败: 文件大小不匹配（差异: ${size2 - size1} 字节）`
      );
      return false;
    }
  } catch (error) {
    console.error("比较文件大小时出错:", error);
    return false;
  }
}

// 分块加密和解密测试
async function runChunkTest() {
  console.log("\n=== 分块加密/解密测试 ===");
  console.log(`使用算法: ${algorithm}`);
  console.log(`分块大小: ${chunkSizeMB} MB`);
  console.log(`测试文件: ${testFile.original}`);

  try {
    ensureFileExists(testFile.original, `测试文件不存在: ${testFile.original}`);

    // 清理旧文件
    cleanupFiles();

    // 开始分块加密
    console.log(
      `\n1. 分块加密文件: ${testFile.original} -> ${testFile.encrypted}`
    );
    console.time("分块加密时间");

    await chunkEncryptFile(
      algorithm,
      key,
      testFile.original,
      testFile.encrypted,
      chunkSizeMB
    );

    console.timeEnd("分块加密时间");
    console.log("加密完成!");

    // 开始分块解密
    console.log(
      `\n2. 分块解密文件: ${testFile.encrypted} -> ${testFile.decrypted}`
    );
    console.time("分块解密时间");

    await chunkDecryptFile(
      algorithm,
      key,
      testFile.encrypted,
      testFile.decrypted
    );

    console.timeEnd("分块解密时间");
    console.log("解密完成!");

    // 验证文件大小
    console.log("\n3. 验证文件完整性:");
    const sizeMatch = await compareFileSizes(
      testFile.original,
      testFile.decrypted
    );

    // 可以选择进行内容验证
    if (sizeMatch) {
      // 使用Rust实现的MD5验证文件内容
      console.log("\n验证文件内容...");
      try {
        console.time("计算MD5时间");

        // 使用Rust实现的MD5函数计算原始文件哈希
        console.log("计算原始文件MD5...");
        const origMd5 = computeFileMd5(testFile.original);
        console.log(`原始文件MD5: ${origMd5}`);

        // 使用Rust实现的MD5函数计算解密后文件哈希
        console.log("计算解密文件MD5...");
        const decryptedMd5 = computeFileMd5(testFile.decrypted);
        console.log(`解密文件MD5: ${decryptedMd5}`);

        console.timeEnd("计算MD5时间");

        // 比较哈希值
        if (origMd5 === decryptedMd5) {
          console.log("✅ 验证成功: 文件内容完全一致");
        } else {
          console.error("❌ 验证失败: 文件内容不一致");
        }
      } catch (err) {
        console.error("文件内容比较出错:", err);
      }
    }
  } catch (error) {
    console.error("分块测试过程中出错:", error);
  }
}

// 执行测试
console.log("开始Rust分块加密/解密测试");
runChunkTest().catch(console.error);
