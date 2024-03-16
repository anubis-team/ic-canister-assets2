use std::str::FromStr;

use candid::CandidType;
use ic_canister_kit::times::now;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};

pub use ic_canister_kit::types::*;

#[allow(unused)]
pub use super::super::{Business, ParsePermission, ScheduleTask};

#[allow(unused)]
pub use super::super::business::*;
#[allow(unused)]
pub use super::business::*;
#[allow(unused)]
pub use super::permission::*;
#[allow(unused)]
pub use super::schedule::schedule_task;

#[allow(unused)]
#[derive(Debug, Clone, Copy, EnumIter, EnumString, strum_macros::Display)]
pub enum RecordTopics {
    // ! 新的权限类型从 0 开始
    Upload = 0, // 上传文件

    // ! 系统倒序排列
    CyclesCharge = 249, // 充值
    Upgrade = 250,      // 升级
    Schedule = 251,     // 定时任务
    Record = 252,       // 记录
    Permission = 253,   // 权限
    Pause = 254,        // 维护
    Initial = 255,      // 初始化
}
#[allow(unused)]
impl RecordTopics {
    pub fn topic(&self) -> RecordTopic {
        *self as u8
    }
    pub fn topics() -> Vec<String> {
        RecordTopics::iter().map(|x| x.to_string()).collect()
    }
    pub fn from(topic: &str) -> Result<Self, strum::ParseError> {
        RecordTopics::from_str(topic)
    }
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone, Default)]
pub struct InnerState {
    pub pause: Pause,             // 记录维护状态
    pub permissions: Permissions, // 记录自身权限
    pub records: Records,         // 记录操作记录
    pub schedule: Schedule,       // 记录定时任务
    // 记录业务数据
    pub business: InnerBusiness,
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone, Default)]
pub struct InnerBusiness {
    pub assets: CoreAssets,
    pub uploading: UploadingAssets,
}

// ============================== 文件数据 ==============================

// 单个文件数据
#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct AssetData {
    pub hash: String,
    pub size: u64,
    pub data: Vec<u8>, // 实际数据
}

// 对外的路径数据 指向文件数据
#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct AssetFile {
    pub path: String,
    pub created: TimestampNanos,
    pub modified: TimestampNanos,
    pub headers: Vec<(String, String)>,
    pub hash: String,
}

// 需要存储的对象
#[derive(CandidType, Serialize, Deserialize, Debug, Clone, Default)]
pub struct CoreAssets {
    pub assets: HashMap<String, AssetData>, // key 是 hash
    pub files: HashMap<String, AssetFile>,  // key 是 path
    hashes: HashMap<String, Vec<String>>, // key 是 hash, value 是 path, 没有 path 的数据是没有保存意义的
}

impl CoreAssets {
    pub fn hash(file: &UploadingFile) -> String {
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(&file.data[0..(file.size as usize)]);
        let digest: [u8; 32] = hasher.finalize().into();
        hex::encode(&digest)
    }
    pub fn put(&mut self, file: &UploadingFile) {
        // 1. 计算 hash
        let hash = CoreAssets::hash(file);
        // 2. 插入 assets: hash -> data
        if !self.assets.contains_key(&hash) {
            let data = (&file.data[0..(file.size as usize)]).to_vec();
            self.assets.insert(
                hash.clone(),
                AssetData {
                    hash: hash.clone(),
                    size: file.size,
                    data,
                },
            );
        }
        // 3. 插入 files: path -> hash
        let now = now();
        if self.files.contains_key(&file.path) {
            let exist = self.files.get_mut(&file.path).unwrap();
            exist.modified = now;
            exist.headers = file.headers.clone();
            exist.hash = hash.clone();
        } else {
            self.files.insert(
                file.path.clone(),
                AssetFile {
                    path: file.path.clone(),
                    created: now,
                    modified: now,
                    headers: file.headers.clone(),
                    hash: hash.clone(),
                },
            );
        }

        // 4. 插入 hashes: hash -> [path]
        if !self.hashes.contains_key(&hash) {
            self.hashes.insert(hash.clone(), vec![]);
        }
        let hash_path = self.hashes.get_mut(&hash).unwrap();
        if !hash_path.contains(&file.path) {
            hash_path.push(file.path.clone());
        }
    }
    pub fn clean(&mut self, path: &String) {
        // 1. 找到文件
        let file = self.files.get(path);
        if let None = file {
            return;
        }
        let file: AssetFile = file.unwrap().clone();
        // 2. 清除 file
        self.files.remove(path);
        // 3. 清除 hashes
        let path_list = self.hashes.get_mut(&file.hash).unwrap();
        let path_list: Vec<String> = path_list
            .clone()
            .into_iter()
            .filter(|p| p != &file.path)
            .collect();
        if path_list.is_empty() {
            // 需要清空
            self.hashes.remove(&file.hash);
            // 4. 清空 assets
            self.assets.remove(&file.hash);
        } else {
            self.hashes.insert(file.hash.clone(), path_list); // 插入新的
        }
    }
    pub fn files(&self) -> Vec<QueryFile> {
        self.files
            .iter()
            .map(|(path, file)| {
                let asset = self.assets.get(&file.hash).unwrap();
                QueryFile {
                    path: path.to_string(),
                    size: asset.size,
                    headers: file.headers.clone(),
                    created: file.created,
                    modified: file.modified,
                    hash: file.hash.clone(),
                }
            })
            .collect()
    }
    pub fn download(&self, path: String) -> Vec<u8> {
        let file = self.files.get(&path).expect("File not found");
        let asset = self.assets.get(&file.hash).expect("File not found");
        asset.data.clone()
    }
    pub fn download_by(&self, path: String, offset: u64, offset_end: u64) -> Vec<u8> {
        let file = self.files.get(&path).expect("File not found");
        let asset = self.assets.get(&file.hash).expect("File not found");
        (&asset.data[(offset as usize)..(offset_end as usize)]).to_vec()
    }
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct QueryFile {
    pub path: String,
    pub size: u64,
    pub headers: Vec<(String, String)>,
    pub created: TimestampNanos,
    pub modified: TimestampNanos,
    pub hash: String,
}

// =========== 上传过程中的对象 ===========

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct UploadingFile {
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub data: Vec<u8>, // 上传中的数据

    pub size: u64,          // 文件大小
    pub chunk_size: u64,    // 块大小
    pub chunks: u32,        // 需要上传的次数
    pub chunked: Vec<bool>, // 记录每一个块的上传状态
}

// 需要存储的对象
#[derive(CandidType, Serialize, Deserialize, Debug, Clone, Default)]
pub struct UploadingAssets {
    files: HashMap<String, UploadingFile>, // key 是 path
}

// 上传参数
#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct UploadingArg {
    pub path: String,
    pub headers: Vec<(String, String)>, // 使用的 header
    pub size: u64,                      // 文件大小
    pub chunk_size: u64,                // 块大小
    pub index: u32,                     // 本次上传的数据
    pub chunk: Vec<u8>,                 // 上传中的数据
}

impl UploadingAssets {
    fn chunks(arg: &UploadingArg) -> u32 {
        let mut chunks = arg.size / arg.chunk_size; // 完整的块数
        if chunks * arg.chunk_size < arg.size {
            chunks += 1;
        }
        chunks as u32
    }
    fn offset(arg: &UploadingArg) -> (usize, usize) {
        let chunks = UploadingAssets::chunks(&arg);
        let offset = arg.chunk_size * arg.index as u64;
        let mut offset_end = offset + arg.chunk_size;
        if arg.index == chunks - 1 {
            offset_end = arg.size;
        }
        (offset as usize, offset_end as usize)
    }
    fn check_arg(arg: &UploadingArg) {
        // 1. 检查 路径名
        assert!(!arg.path.is_empty(), "must has path");
        assert!(arg.path.starts_with("/"), "path must start with /");
        // 2. 检查 headers
        // 3. 检查 size
        assert!(0 < arg.size, "size can not be 0");
        assert!(
            arg.size <= 1024 * 1024 * 1024 * 4, // 最大文件 4G
            "size must less than 4GB"
        );
        // 4. 检查 chunk_size
        assert!(0 < arg.chunk_size, "chunk size can not be 0");
        // 5. 检查 index
        let chunks = UploadingAssets::chunks(&arg);
        assert!(arg.index < chunks, "wrong index");
        // 6. 检查 data
        if arg.index < chunks - 1 || arg.size == arg.chunk_size * chunks as u64 {
            // 是前面完整的 或者 整好整除
            assert!(
                arg.chunk.len() as u64 == arg.chunk_size,
                "wrong chunk length"
            );
        } else {
            // 是剩下的
            assert!(
                arg.chunk.len() as u64 == arg.size % arg.chunk_size,
                "wrong chunk length"
            );
        }
    }
    fn check_file(&mut self, arg: &UploadingArg) {
        if self.files.contains_key(&arg.path) {
            // 已经有这个文件了, 需要比较一下, 参数是否一致
            let file = self.files.get(&arg.path).unwrap();
            assert!(arg.path == file.path, "wrong path, system error.");
            let chunks = UploadingAssets::chunks(&arg);
            if arg.size != file.size // 文件长度不一致
                || file.data.len() < file.size as usize // 暂存长度不对
                || arg.chunk_size != file.chunk_size
                || chunks != file.chunks
                || file.chunked.len() < file.chunks as usize
            {
                // 非致命错误, 清空原来的文件就好
                self.files.remove(&arg.path);
            }
        }
        if !self.files.contains_key(&arg.path) {
            // 原来没有的情况下
            let chunks = UploadingAssets::chunks(&arg);
            self.files.insert(
                arg.path.clone(),
                UploadingFile {
                    path: arg.path.clone(),
                    headers: arg.headers.clone(),
                    data: vec![0; arg.size as usize],
                    size: arg.size,
                    chunk_size: arg.chunk_size,
                    chunks,
                    chunked: vec![false; chunks as usize],
                },
            );
        }
    }
    pub fn put(&mut self, arg: UploadingArg) -> Option<&UploadingFile> {
        // 0. 检查参数是否有效
        UploadingAssets::check_arg(&arg);

        // 1. 检查文件
        self.check_file(&arg);

        // 2. 找的对应的缓存文件
        let file = self.files.get_mut(&arg.path).unwrap();

        // 3. 复制有效的信息
        let (offset, offset_end) = UploadingAssets::offset(&arg);
        file.headers = arg.headers;
        file.data.splice(offset..offset_end, arg.chunk); // 复制内容
        file.chunked[arg.index as usize] = true;

        // 4. 是否已经完整
        for uploaded in file.chunked.iter() {
            if !uploaded {
                return None; // 还有没上传的
            }
        }
        Some(file) // 已经完成的
    }
    pub fn clean(&mut self, path: &String) {
        self.files.remove(path);
    }
}
