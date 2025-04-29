const path = require("path");
const {
  encryptFile,
  decryptFile,
  chunkEncryptFile,
  chunkDecryptFile,
  getFileSize,
  computeFileMd5,
} = require("../");

let key = "9d88ef5de5927ad5cf19e8f1f57d6e9955c433c091ca747b0fb739d817a06e6f";

// 加密算法（可选 'aes' 或 'chacha20poly1305'）
const algorithm = "chacha20poly1305";

// 加密文件
const encryptedFile = (filepath, destpath) => {
  const result = encryptFile(
    algorithm,
    Buffer.from(key, "hex"),
    filepath,
    destpath
  );

  console.log("加密结果:", result);
  console.log("文件大小(KB):", result.fileSize);
  return result;
};

// 解密文件
const DecryptFile = (filepath, destpath) => {
  const result = decryptFile(
    algorithm,
    Buffer.from(key, "hex"),
    filepath,
    destpath
  );
  console.log("解密结果:", result);
  console.log("解密后文件大小(KB):", result.fileSize);
  console.log("加密文件大小(KB):", result.encryptedSize);
  return result;
};

// 处理大文件的加密
const lagerEncrypted = async (filepath, destpath, chunkSize) => {
  console.time("分块加密时间");
  const result = await chunkEncryptFile(
    algorithm,
    Buffer.from(key, "hex"),
    filepath,
    destpath,
    chunkSize
  );
  console.timeEnd("分块加密时间");
  console.log("分块加密结果:", result);
  console.log("文件大小(KB):", result.fileSize);
  console.log("块大小(KB):", result.chunkSize);
  console.log("总块数:", result.totalChunks);
  return result;
};

const largetDecryptFile = async (filepath, destpath) => {
  console.time("分块解密时间");
  const result = await chunkDecryptFile(
    algorithm,
    Buffer.from(key, "hex"),
    filepath,
    destpath
  );
  console.timeEnd("分块解密时间");
  console.log("分块解密结果:", result);
  console.log("原始文件大小(KB):", result.originalSizeKB);
  console.log("总读取字节(KB):", result.totalBytesKB);
  console.log("块大小(KB):", result.chunkSizeKB);
  console.log("总块数:", result.totalChunks);
  return result;
};

// 文件大小
const getFileSizeExample = (filepath) => {
  const size = getFileSize(filepath);
  console.log(`文件 ${filepath} 的大小为: ${size}KB`);
  return size;
};

// 计算MD5
const computeMd5 = (filepath) => {
  const hash = computeFileMd5(filepath);
  console.log(`文件 ${filepath} 的MD5: ${hash}`);
  return hash;
};

// 注释掉默认执行的示例
// encryptedFile(
//   path.join(__dirname, "./test.txt"),
//   path.join(__dirname, "./test.vkk")
// );
// DecryptFile(
//   path.join(__dirname, "./test.vkk"),
//   path.join(__dirname, "./test-dect.txt")
// );

// 导出所有方法
module.exports = {
  encryptedFile,
  DecryptFile,
  lagerEncrypted,
  largetDecryptFile,
  getFileSizeExample,
  computeMd5,
};

// 示例用法
(async () => {
  // 示例: 获取文件大小
  // getFileSizeExample(path.join(__dirname, "./test.txt"));
  // 示例: 计算文件MD5
  // computeMd5(path.join(__dirname, "./test.txt"));
  // 示例: 加密解密大文件
  // await lagerEncrypted(
  //   path.join(__dirname, "./video.mp4"),
  //   path.join(__dirname, "./video.wocao"),
  //   10
  // );
  // await largetDecryptFile(
  //   path.join(__dirname, "./video.wocao"),
  //   path.join(__dirname, "./video2.mp4")
  // );
})();
