use super::*;
#[allow(unused)]
pub use ic_canister_kit::identity::self_canister_id;
#[allow(unused)]
pub use ic_canister_kit::types::{CanisterId, PauseReason, UserId};
#[allow(unused)]
pub use std::collections::{HashMap, HashSet};
#[allow(unused)]
pub use std::fmt::Display;

pub trait Business:
    Pausable<PauseReason>
    + ParsePermission
    + Permissable<Permission>
    + Recordable<Record, RecordTopic, RecordSearch>
    + Schedulable
    + ScheduleTask
    + StableHeap
{
    fn business_files(&self) -> Vec<QueryFile>;
    fn business_download(&self, path: String) -> Vec<u8>;
    fn business_download_by(&self, path: String, offset: u64, offset_end: u64) -> Vec<u8>;

    fn business_upload(&mut self, args: Vec<UploadingArg>);

    fn business_delete(&mut self, names: Vec<String>);

    fn business_assets_files(&self) -> &HashMap<String, AssetFile>;
    fn business_assets_assets(&self) -> &HashMap<HashDigest, AssetData>;
}

// 业务实现
impl Business for State {
    fn business_files(&self) -> Vec<QueryFile> {
        self.get().business_files()
    }
    fn business_download(&self, path: String) -> Vec<u8> {
        self.get().business_download(path)
    }
    fn business_download_by(&self, path: String, offset: u64, offset_end: u64) -> Vec<u8> {
        self.get().business_download_by(path, offset, offset_end)
    }

    fn business_upload(&mut self, args: Vec<UploadingArg>) {
        self.get_mut().business_upload(args)
    }

    fn business_delete(&mut self, names: Vec<String>) {
        self.get_mut().business_delete(names)
    }

    fn business_assets_files(&self) -> &HashMap<String, AssetFile> {
        self.get().business_assets_files()
    }
    fn business_assets_assets(&self) -> &HashMap<HashDigest, AssetData> {
        self.get().business_assets_assets()
    }
}
