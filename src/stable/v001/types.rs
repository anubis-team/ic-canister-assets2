use std::str::FromStr;

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
    UploadFile = 0, // 上传文件
    DeleteFile = 1, // 删除文件

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

// 框架需要的数据结构
#[derive(Serialize, Deserialize, Default)]
pub struct CanisterKit {
    pub pause: Pause,             // 记录维护状态 // ? 堆内存 序列化
    pub permissions: Permissions, // 记录自身权限 // ? 堆内存 序列化
    pub records: Records,         // 记录操作记录 // ? 堆内存 序列化
    pub schedule: Schedule,       // 记录定时任务 // ? 堆内存 序列化
}

// 能序列化的和不能序列化的放在一起
// 其中不能序列化的采用如下注解
// #[serde(skip)] 默认初始化方式
// #[serde(skip, default="init_xxx")] 指定初始化方式
// ! 如果使用 ic-stable-structures 提供的稳定内存，不能变更 memory_id 的使用类型，否则会出现各个版本不兼容，数据会被清空
#[derive(Serialize, Deserialize)]
pub struct InnerState {
    pub canister_kit: CanisterKit, // 框架需要的数据 // ? 堆内存 序列化

    // 业务数据
    pub hashed: bool, // 是否相信上传的 hash 值，true -> 直接采用接口传递的 hash 值， false -> 数据上传完成后，需要罐子再 hash 一次 // ? 堆内存 序列化

    pub assets: HashMap<HashDigest, AssetData>, // key 是 hash // ? 堆内存 序列化
    pub files: HashMap<String, AssetFile>,      // key 是 path // ? 堆内存 序列化
    hashes: HashMap<HashDigest, HashedPath>, // key 是 hash, value 是 path, 没有 path 的数据是没有保存意义的 // ? 堆内存 序列化

    uploading: HashMap<String, UploadingFile>, // key 是 path // ? 堆内存 序列化
}

impl Default for InnerState {
    fn default() -> Self {
        ic_cdk::println!("InnerState::default()");
        Self {
            canister_kit: Default::default(),

            // 业务数据
            hashed: Default::default(),

            assets: Default::default(),
            files: Default::default(),
            hashes: Default::default(),

            uploading: Default::default(),
        }
    }
}

use candid::CandidType;

// ============================== 文件数据 ==============================

#[derive(
    CandidType, Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct HashDigest([u8; 32]);

impl HashDigest {
    pub fn hex(&self) -> String {
        hex::encode(self.0)
    }
}

mod assets {
    use candid::CandidType;
    use serde::{Deserialize, Serialize};

    use super::HashDigest;

    // 单个文件数据
    #[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
    pub struct AssetData {
        data: Vec<u8>, // 实际数据
    }

    impl AssetData {
        pub fn from(_hash: &HashDigest, data: Vec<u8>) -> Self {
            Self { data }
        }
        pub fn slice(
            &self,
            _hash: &HashDigest,
            data_size: u64,
            offset: usize,
            size: usize,
        ) -> std::borrow::Cow<'_, [u8]> {
            assert!(offset < data_size as usize);
            let offset_end = offset + size;
            assert!(offset_end <= data_size as usize);
            std::borrow::Cow::Borrowed(&self.data[offset..offset_end])
        }
    }
}

pub use assets::AssetData;

// 对外的路径数据 指向文件数据
#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct AssetFile {
    pub path: String,
    pub created: TimestampNanos,
    pub modified: TimestampNanos,
    pub headers: Vec<(String, String)>,
    pub hash: HashDigest,
    pub size: u64,
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone, Default)]
pub struct HashedPath(HashSet<String>);

// =========== 上传过程中的对象 ===========

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct UploadingFile {
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub hash: HashDigest, // hash 值，在 hashed 为 false 的情况下不使用
    pub data: Vec<u8>,    // 上传中的数据

    pub size: u64,          // 文件大小
    pub chunk_size: u32,    // 块大小 块分割的大小
    pub chunks: u32,        // 需要上传的次数
    pub chunked: Vec<bool>, // 记录每一个块的上传状态
}

// 上传参数
#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct UploadingArg {
    pub path: String,
    pub headers: Vec<(String, String)>, // 使用的 header
    pub hash: HashDigest,               // hash 值，在 hashed 为 false 的情况下不使用
    pub size: u64,                      // 文件大小
    pub chunk_size: u32,                // 块大小 块分割的大小
    pub index: u32,                     // 本次上传的数据
    pub chunk: Vec<u8>,                 // 上传中的数据
}

// =========== 查询的对象 ===========

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct QueryFile {
    pub path: String,
    pub size: u64,
    pub headers: Vec<(String, String)>,
    pub created: TimestampNanos,
    pub modified: TimestampNanos,
    pub hash: String,
}

impl InnerState {
    fn hash(file: &UploadingFile) -> HashDigest {
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(&file.data[0..(file.size as usize)]);
        let digest: [u8; 32] = hasher.finalize().into();
        HashDigest(digest)
    }
    fn put_file(
        &mut self,
        path: String,
        headers: Vec<(String, String)>,
        hash: HashDigest,
        size: u64,
    ) {
        // 3. 插入 files: path -> hash
        let now = ic_canister_kit::times::now();
        if let Some(exist) = self.files.get_mut(&path) {
            exist.modified = now;
            exist.headers = headers;
            exist.hash = hash;
        } else {
            self.files.insert(
                path.clone(),
                AssetFile {
                    path: path.clone(),
                    created: now,
                    modified: now,
                    headers,
                    hash,
                    size,
                },
            );
        }

        // 4. 插入 hashes: hash -> [path]
        self.hashes.entry(hash).or_default();
        if let Some(hash_path) = self.hashes.get_mut(&hash) {
            hash_path.0.insert(path);
        }
    }
    fn put_assets(&mut self, file: UploadingFile) {
        // 0. 先清空同路径的文件
        self.clean_file(&file.path);
        // 1. 计算 hash
        let hash = if self.hashed {
            file.hash // hashed true 直接使用
        } else {
            Self::hash(&file) // hashed false 要计算一次
        };
        // 2. 插入 assets: hash -> data
        self.assets
            .entry(hash)
            .or_insert_with(|| AssetData::from(&hash, file.data));

        self.put_file(file.path, file.headers, hash, file.size); // 存完毕 assets 数据了，然后要对文件建立代理索引
    }
    pub fn clean_file(&mut self, path: &String) {
        // 1. 删除文件
        let file = match self.files.remove(path) {
            Some(file) => file.clone(),
            None => return,
        };
        // 2. 清除 hashes
        if let Some(HashedPath(path_set)) = self.hashes.get_mut(&file.hash) {
            path_set.remove(&file.path);
            if path_set.is_empty() {
                // 需要清空
                self.hashes.remove(&file.hash);
                // 4. 清空 assets
                self.assets.remove(&file.hash);
            }
        }
    }
    pub fn files(&self) -> Vec<QueryFile> {
        self.files
            .iter()
            .map(|(path, file)| QueryFile {
                path: path.to_string(),
                size: file.size,
                headers: file.headers.clone(),
                created: file.created,
                modified: file.modified,
                hash: file.hash.hex(),
            })
            .collect()
    }
    pub fn download(&self, path: String) -> Vec<u8> {
        #[allow(clippy::expect_used)] // ? SAFETY
        let file = self.files.get(&path).expect("File not found");
        #[allow(clippy::expect_used)] // ? SAFETY
        let asset = self.assets.get(&file.hash).expect("File not found");
        asset
            .slice(&file.hash, file.size, 0, file.size as usize)
            .to_vec()
    }
    pub fn download_by(&self, path: String, offset: u64, size: u64) -> Vec<u8> {
        #[allow(clippy::expect_used)] // ? SAFETY
        let file = self.files.get(&path).expect("File not found");
        #[allow(clippy::expect_used)] // ? SAFETY
        let asset = self.assets.get(&file.hash).expect("File not found");
        asset
            .slice(&file.hash, file.size, offset as usize, size as usize)
            .to_vec()
    }

    fn chunks(arg: &UploadingArg) -> u32 {
        let mut chunks = arg.size / arg.chunk_size as u64; // 完整的块数
        if chunks * (arg.chunk_size as u64) < arg.size {
            chunks += 1;
        }
        chunks as u32
    }
    fn offset(arg: &UploadingArg) -> (usize, usize) {
        let chunks = Self::chunks(arg);
        let offset = arg.chunk_size as u64 * arg.index as u64;
        let mut offset_end = offset + arg.chunk_size as u64;
        if arg.index == chunks - 1 {
            offset_end = arg.size;
        }
        (offset as usize, offset_end as usize)
    }
    fn check_path_and_headers(arg: &UploadingArg) {
        // 1. 检查 路径名
        assert!(!arg.path.is_empty(), "must has path");
        assert!(arg.path.starts_with('/'), "path must start with /");
        // 2. 检查 headers
        for (name, value) in &arg.headers {
            assert!(name.len() <= 64, "header name is too large");
            assert!(value.len() <= 1024 * 8, "header value is too large");
        }
    }
    fn check_size_and_data(arg: &UploadingArg) {
        // 3. 检查 size
        assert!(0 < arg.size, "size can not be 0");
        assert!(
            arg.size <= 1024 * 1024 * 1024 * 2, // 最大文件 2G
            "size must less than 4GB"
        );
        // 4. 检查 chunk_size
        assert!(0 < arg.chunk_size, "chunk size can not be 0");
        // 5. 检查 index
        let chunks = Self::chunks(arg);
        assert!(arg.index < chunks, "wrong index");
        // 6. 检查 data
        if arg.index < chunks - 1 || arg.size == arg.chunk_size as u64 * chunks as u64 {
            // 是前面完整的 或者 整好整除
            assert!(
                arg.chunk.len() as u32 == arg.chunk_size,
                "wrong chunk length"
            );
        } else {
            // 是剩下的
            assert!(
                arg.chunk.len() as u64 == arg.size % (arg.chunk_size as u64),
                "wrong chunk length"
            );
        }
    }
    fn assure_uploading(&mut self, arg: &UploadingArg) {
        let chunks = Self::chunks(arg);
        if let Some(exist) = self.uploading.get(&arg.path) {
            // 已经有这个文件了, 需要比较一下, 参数是否一致
            assert!(exist.path == arg.path, "wrong path, system error.");
            if exist.hash != arg.hash // hash 不一致
                || exist.size != arg.size // 文件长度不一致
                || exist.data.len() != arg.size as usize // 暂存长度不对
                || exist.chunk_size != arg.chunk_size
                || exist.chunks != chunks
                || exist.chunked.len() != chunks as usize
            {
                // 非致命错误, 清空原来的文件就好
                self.files.remove(&arg.path);
            }
        } else {
            // 原来没有的情况下
            self.uploading.insert(
                arg.path.clone(),
                UploadingFile {
                    path: arg.path.clone(),
                    headers: arg.headers.clone(),
                    hash: arg.hash,
                    data: vec![0; arg.size as usize],
                    size: arg.size,
                    chunk_size: arg.chunk_size,
                    chunks,
                    chunked: vec![false; chunks as usize],
                },
            );
        }
    }
    pub fn put_uploading(&mut self, arg: UploadingArg) {
        // 1. 检查参数是否有效
        Self::check_path_and_headers(&arg);

        // 2. 如果 hashed true 并且已经存在改 hash 值文件了，直接保存即可
        if self.hashed && self.assets.contains_key(&arg.hash) {
            if let Some(path) = self.hashes.get(&arg.hash) {
                if let Some(path) = path.0.iter().next() {
                    if let Some(file) = self.files.get(path) {
                        self.put_file(arg.path, arg.headers, arg.hash, file.size); // size 不可信，只能从已存在的文件内容中查找
                        return;
                    }
                }
            }
        }

        // 3. 检查其他参数
        Self::check_size_and_data(&arg);

        // 4. 确保有缓存空间
        self.assure_uploading(&arg); // 确保该文件已经存在缓存数据了

        // 5. 找的对应的缓存文件
        let mut done = false;
        if let Some(file) = self.uploading.get_mut(&arg.path) {
            // 3. 复制有效的信息
            let (offset, offset_end) = Self::offset(&arg);
            file.headers = arg.headers;
            file.data.splice(offset..offset_end, arg.chunk); // 复制内容
            file.chunked[arg.index as usize] = true;

            // 4. 是否已经完整
            done = file.chunked.iter().all(|c| *c);
        }
        if done {
            if let Some(file) = self.uploading.remove(&arg.path) {
                // 处理这个已经完成的数据
                self.put_assets(file);
            }
        }
    }
    pub fn clean_uploading(&mut self, path: &String) {
        self.files.remove(path);
    }
}
