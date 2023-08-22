use std::collections::HashMap;

use candid::{CandidType, Deserialize};
use ic_canister_kit::types::Stable;

// 单个文件数据
#[derive(CandidType, Deserialize, Default, Debug, Clone)]
pub struct AssetData {
    pub hash: String,
    pub data: Vec<u8>, // 实际数据
}

// 对外的路径数据 指向文件数据
#[derive(CandidType, Deserialize, Default, Debug, Clone)]
pub struct AssetFile {
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub hash: String,
}

// 需要存储的对象
#[derive(Default, Debug, Clone)]
pub struct CoreAssets {
    assets: HashMap<String, AssetData>, // key 是 hash
    files: HashMap<String, AssetFile>,  // key 是 path
}

pub type CoreAssetsState = (Vec<AssetData>, Vec<AssetFile>);

impl Stable<CoreAssetsState, CoreAssetsState> for CoreAssets {
    fn store(&mut self) -> CoreAssetsState {
        let assets = std::mem::take(&mut self.assets);
        let assets = assets.into_iter().map(|(_, asset)| asset).collect();
        let files = std::mem::take(&mut self.files);
        let files = files.into_iter().map(|(_, file)| file).collect();
        (assets, files)
    }

    fn restore(&mut self, restore: CoreAssetsState) {
        let assets = restore.0;
        let assets = assets
            .into_iter()
            .map(|asset| (asset.hash.clone(), asset))
            .collect();
        let files = restore.1;
        let files = files
            .into_iter()
            .map(|file| (file.path.clone(), file))
            .collect();
        let _ = std::mem::replace(&mut self.assets, assets);
        let _ = std::mem::replace(&mut self.files, files);
    }
}

// =========== 上传过程中的对象 ===========

#[derive(CandidType, Deserialize, Default, Debug, Clone)]
pub struct UploadingFile {
    pub path: String,
    pub data: Vec<u8>, // 上传中的数据
}

// 需要存储的对象
#[derive(Default, Debug, Clone)]
pub struct UploadingAssets {
    files: HashMap<String, UploadingFile>, // key 是 path
}

pub type UploadingAssetsState = (Vec<UploadingFile>,);

impl Stable<UploadingAssetsState, UploadingAssetsState> for UploadingAssets {
    fn store(&mut self) -> UploadingAssetsState {
        let files = std::mem::take(&mut self.files);
        let files = files.into_iter().map(|(_, file)| file).collect();
        (files,)
    }

    fn restore(&mut self, restore: UploadingAssetsState) {
        let files = restore.0;
        let files = files
            .into_iter()
            .map(|file| (file.path.clone(), file))
            .collect();
        let _ = std::mem::replace(&mut self.files, files);
    }
}
